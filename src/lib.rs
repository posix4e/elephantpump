use std::ffi::{CStr, CString};
use std::fmt;
use std::mem::size_of;
#[cfg(feature = "pg-ldc-messages")]
use std::ops::Deref;
#[cfg(feature = "pg-ldc-messages")]
use std::slice::from_raw_parts;

#[cfg(feature = "pg-ldc-messages")]
extern crate base64;
extern crate libc;
extern crate rpgffi as pg;
#[cfg(feature = "pg-ldc-messages")]
extern crate serde_json;


macro_rules! log {
    ($msg:expr) => {
        elog(file!(), line!(), "log()", $msg)
    }
}


// Implementation of initialization and callbacks.

pub unsafe extern "C" fn init(cb: *mut pg::OutputPluginCallbacks) {
    (*cb).startup_cb = Some(startup);
    (*cb).begin_cb = Some(begin);
    (*cb).change_cb = Some(change);
    (*cb).commit_cb = Some(commit);
    (*cb).shutdown_cb = Some(shutdown);
    #[cfg(feature = "pg-ldc-messages")]
    {
        (*cb).message_cb = Some(message);
    }
}

unsafe extern "C" fn startup(
    ctx: *mut pg::LogicalDecodingContext,
    options: *mut pg::OutputPluginOptions,
    _is_init: pg::bool_,
) {
    let last_relid = pg::palloc0(size_of::<pg::Oid>());
    (*ctx).output_plugin_private = last_relid;
    (*options).output_type =
        pg::OutputPluginOutputType::OUTPUT_PLUGIN_TEXTUAL_OUTPUT;
}

unsafe extern "C" fn begin(
    ctx: *mut pg::LogicalDecodingContext,
    txn: *mut pg::ReorderBufferTXN,
) {
    let s = CString::new("{ \"begin\": %u }").unwrap();
    pg::OutputPluginPrepareWrite(ctx, CTRUE);
    pg::appendStringInfo((*ctx).out, s.as_ptr(), (*txn).xid);
    pg::OutputPluginWrite(ctx, CTRUE);
}

unsafe extern "C" fn change(
    ctx: *mut pg::LogicalDecodingContext,
    _txn: *mut pg::ReorderBufferTXN,
    relation: pg::Relation,
    change: *mut pg::ReorderBufferChange,
) {
    let relid = (*relation).rd_id;
    let last_relid: *mut pg::Oid = (*ctx).output_plugin_private as
        *mut pg::Oid;
    if *last_relid != relid {
        pg::OutputPluginPrepareWrite(ctx, CFALSE);
        append_schema(relation, (*ctx).out);
        pg::OutputPluginWrite(ctx, CFALSE);
        *last_relid = relid;
    }
    pg::OutputPluginPrepareWrite(ctx, CTRUE);
    append_change(relation, change, (*ctx).out);
    pg::OutputPluginWrite(ctx, CTRUE);
}

unsafe extern "C" fn commit(
    ctx: *mut pg::LogicalDecodingContext,
    txn: *mut pg::ReorderBufferTXN,
    _lsn: pg::XLogRecPtr,
) {
    let s = CString::new("{ \"commit\": %u, \"t\": \"%s\" }").unwrap();
    let t = pg::timestamptz_to_str((*txn).commit_time);
    pg::OutputPluginPrepareWrite(ctx, CTRUE);
    pg::appendStringInfo((*ctx).out, s.as_ptr(), (*txn).xid, t);
    pg::OutputPluginWrite(ctx, CTRUE);
    let last_relid: *mut pg::Oid = (*ctx).output_plugin_private as
        *mut pg::Oid;
    *last_relid = 0;
}

unsafe extern "C" fn shutdown(ctx: *mut pg::LogicalDecodingContext) {
    pg::pfree((*ctx).output_plugin_private);
}

#[cfg(feature = "pg-ldc-messages")]
unsafe extern "C" fn message(
    ctx: *mut pg::LogicalDecodingContext,
    _txn: *mut pg::ReorderBufferTXN,
    _lsn: pg::XLogRecPtr,
    transactional: pg::bool_,
    prefix: *const std::os::raw::c_char,
    message_size: pg::Size,
    message: *const std::os::raw::c_char,
) {
    pg::OutputPluginPrepareWrite(ctx, CTRUE);
    append_message(transactional, prefix, message_size, message, (*ctx).out);
    pg::OutputPluginWrite(ctx, CTRUE);
}

trait PGAppend<T> {
    unsafe fn add_str(self, T);
    unsafe fn add_json(self, T);
}

impl<'a> PGAppend<&'a str> for pg::StringInfo {
    unsafe fn add_str(self, t: &'a str) {
        pg::appendStringInfoString(self, CString::new(t).unwrap().as_ptr());
    }
    unsafe fn add_json(self, t: &'a str) {
        pg::escape_json(self, CString::new(t).unwrap().as_ptr());
    }
}

impl PGAppend<*mut i8> for pg::StringInfo {
    unsafe fn add_str(self, t: *mut i8) { self.add_str(t as *const i8); }
    unsafe fn add_json(self, t: *mut i8) { self.add_json(t as *const i8); }
}

impl PGAppend<*const i8> for pg::StringInfo {
    unsafe fn add_str(self, t: *const i8) {
        pg::appendStringInfoString(self, t);
    }
    unsafe fn add_json(self, t: *const i8) { pg::escape_json(self, t); }
}

#[cfg(feature = "pg-ldc-messages")]
struct OutputBytesInMostFriendlyWay(*const u8, usize);

#[cfg(feature = "pg-ldc-messages")]
impl PGAppend<OutputBytesInMostFriendlyWay> for pg::StringInfo {
    unsafe fn add_str(self, t: OutputBytesInMostFriendlyWay) {
        let bytes: &[u8] = from_raw_parts(t.0, t.1);
        let decoded = String::from_utf8_lossy(bytes);
        self.add_str(decoded.deref());
    }
    unsafe fn add_json(self, t: OutputBytesInMostFriendlyWay) {
        let bytes: &[u8] = from_raw_parts(t.0, t.1);
        // Although null is allowed in a string, Postgres's internal JSON
        // functions do not work well with it (see `escape_json`, which takes a
        // C-string) and Postgres does not allow conversion from `bytea` to
        // `text` where the `bytea` contains null.
        if !bytes.contains(&0) {
            if let Ok(ref s) = String::from_utf8(bytes.to_vec()) {
                if s.trim_left().starts_with("{") {
                    let parsed: serde_json::Result<serde_json::Value> =
                        serde_json::from_str(s);
                    if let Ok(ref json) = parsed {
                        self.add_str(json.to_string().deref());
                        return;
                    }
                }
                // Definitely a string.
                self.add_json(s.deref());
                return;
            }
        }
        // Definitely binary.
        let as_base64: String = base64::encode(bytes);
        self.add_str("{");
        self.add_json("base64");
        self.add_str(":");
        self.add_json(as_base64.deref());
        self.add_str("}");
    }
}


struct BufferChangeWrapper(pg::ReorderBufferChangeType);

impl fmt::Display for BufferChangeWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use pg::ReorderBufferChangeType::*;
        #[allow(unreachable_patterns)]
        let formatted_token = match self.0 {
            REORDER_BUFFER_CHANGE_INSERT => "insert",
            REORDER_BUFFER_CHANGE_UPDATE => "update",
            REORDER_BUFFER_CHANGE_DELETE => "delete",
            REORDER_BUFFER_CHANGE_INTERNAL_SNAPSHOT => "internal_snapshot",
            REORDER_BUFFER_CHANGE_INTERNAL_COMMAND_ID => "internal_command_id",
            REORDER_BUFFER_CHANGE_INTERNAL_TUPLECID => "internal_tuplecid",
            #[cfg_attr(rustfmt, rustfmt_skip)]
            REORDER_BUFFER_CHANGE_INTERNAL_SPEC_INSERT =>
                "internal_spec_insert",
            _ => "unknown_change_type",   // NB: Unreachable after Postgres 9.4
        };
        write!(f, "{}", formatted_token)
    }
}

unsafe fn append_change(
    relation: pg::Relation,
    change: *mut pg::ReorderBufferChange,
    out: pg::StringInfo,
) {
    use pg::ReorderBufferChangeType::*;
    let relid = (*relation).rd_id;
    let name = pg::get_rel_name(relid);
    let ns = pg::get_namespace_name(pg::get_rel_namespace(relid));
    let qualified_name = pg::quote_qualified_identifier(ns, name);
    let tuple_desc = (*relation).rd_att;
    let tuples = (*change).data.tp;
    let tuple_new = tuples.newtuple;
    let tuple_old = tuples.oldtuple;
    let token = match (*change).action {
        REORDER_BUFFER_CHANGE_INSERT => "insert",
        REORDER_BUFFER_CHANGE_UPDATE => "update",
        REORDER_BUFFER_CHANGE_DELETE => "delete",
        _ => "unrecognized",
    };
    if token == "unrecognized" {
        log!(
            format!(
                "Unrecognized Change Action: [ {} ]",
                BufferChangeWrapper((*change).action)
            )
            .as_str()
        );
        return;
    }
    out.add_str("{ ");
    out.add_json(token);
    out.add_str(": ");
    append_tuple_buf_as_json(tuple_new, tuple_desc, out);
    if !tuple_old.is_null() {
        out.add_str(", \"@\": ");
        append_tuple_buf_as_json(tuple_old, tuple_desc, out);
    }
    out.add_str(", ");
    out.add_json("table");
    out.add_str(": ");
    out.add_json(qualified_name);
    out.add_str(" }");
}


unsafe fn append_tuple_buf_as_json(
    data: *mut pg::ReorderBufferTupleBuf,
    desc: pg::TupleDesc,
    out: pg::StringInfo,
) {
    if !data.is_null() {
        let heap_tuple = &mut (*data).tuple;
        let n = (*desc).natts as usize;
        let attrs = (*desc).attrs;

        // Pull out every single field to check for stale TOAST.
        let mut datums: Vec<pg::Datum> = Vec::new();
        let mut nulls: Vec<pg::bool_> = Vec::new();
        datums.resize(n, 0);
        nulls.resize(n, CFALSE);
        pg::heap_deform_tuple(
            heap_tuple,
            desc,
            datums.as_mut_ptr(),
            nulls.as_mut_ptr(),
        );

        let mut skip: Vec<pg::Form_pg_attribute> = Vec::with_capacity(n);

        for i in 0..n {
            let datum: pg::Datum = datums[i];
            let attr = *attrs.offset(i as isize);
            if datum == 0 || (*attr).attnum <= 0 {
                continue;
            }
            if (*attr).attisdropped == CFALSE && is_stale_toast(datum, attr) {
                skip.push(attr);
                // Mark as NULL to trick heap_form_tuple().
                nulls[i] = CTRUE;
            }
        }

        // Mark as dropped to trick row_to_json().
        for attr in &mut skip {
            (**attr).attisdropped = CTRUE;
        }

        let new =
            pg::heap_form_tuple(desc, datums.as_mut_ptr(), nulls.as_mut_ptr());

        let datum = pg::heap_copy_tuple_as_datum(new, desc);
        let empty_oid: pg::Oid = 0;
        let json =
            pg::DirectFunctionCall1Coll(Some(row_to_json), empty_oid, datum);

        // Set back to true because who knows how else these attrs, which are
        // part of the passed in tuple description, are being used.
        for attr in &mut skip {
            (**attr).attisdropped = CFALSE;
        }

        let ptr = json as *const pg::varlena;
        let text = pg::text_to_cstring(ptr);
        pg::appendStringInfoString(out, text);

        if skip.len() > 0 {
            out.add_str(", ");
            out.add_json("skipped");
            out.add_str(": [");
            for (i, attr) in skip.into_iter().enumerate() {
                if i > 0 {
                    out.add_str(", ");
                }
                out.add_json((*attr).attname.data.as_mut_ptr());
            }
            out.add_str("]");
        }
    } else {
        out.add_str("{}");
    }
}

unsafe fn append_schema(relation: pg::Relation, out: pg::StringInfo) {
    let relid = (*relation).rd_id;
    let tupdesc = (*relation).rd_att;
    let name = pg::get_rel_name(relid);
    let ns = pg::get_namespace_name(pg::get_rel_namespace(relid));
    let qualified_name = pg::quote_qualified_identifier(ns, name);
    out.add_str("{ ");
    out.add_json("schema");
    out.add_str(": ");
    out.add_str("[");
    let mut first: bool = true;
    for i in 0..(*tupdesc).natts {
        let attr = *(*tupdesc).attrs.offset(i as isize);
        let num = (*attr).attnum;
        if (*attr).attisdropped == CTRUE || num <= 0 {
            continue;
        }
        let col = pg::get_attname(relid, num);
        let typ = pg::format_type_be(pg::get_atttype(relid, num));
        if !first {
            out.add_str(",");
        } else {
            first = false;
        }
        out.add_str("{");
        out.add_json(col);
        out.add_str(":");
        out.add_json(typ);
        out.add_str("}");
    }
    out.add_str("]");
    out.add_str(", ");
    out.add_json("table");
    out.add_str(": ");
    out.add_json(qualified_name);
    out.add_str(" }");
}

#[cfg(feature = "pg-ldc-messages")]
unsafe fn append_message(
    transactional: pg::bool_,
    prefix: *const std::os::raw::c_char,
    message_size: pg::Size,
    message: *const std::os::raw::c_char,
    out: pg::StringInfo,
) {
    let data = OutputBytesInMostFriendlyWay(
        message as *const u8,
        message_size as usize,
    );

    out.add_str("{ ");

    out.add_json("prefix");
    out.add_str(": ");
    out.add_json(prefix);
    out.add_str(", ");

    out.add_json("message");
    out.add_str(": ");
    out.add_json(data);
    out.add_str(", ");

    out.add_json("transactional");
    out.add_str(": ");

    if transactional == CTRUE {
        out.add_str("true");
    } else {
        out.add_str("false");
    }

    out.add_str(" }");
}

extern "C" fn row_to_json(fcinfo: pg::FunctionCallInfo) -> pg::Datum {
    // We wrap the unsafe call to make it safe, so that it can be passed as
    // a function pointer to DirectFunctionCall1Coll(). This is a spurious
    // artifact of the generated binding.
    unsafe { pg::row_to_json(fcinfo) }
}

/* This is a simulation of `VARATT_IS_EXTERNAL_ONDISK`.

```c
#define VARATT_IS_EXTERNAL_ONDISK(PTR) \
  (VARATT_IS_EXTERNAL(PTR) && VARTAG_EXTERNAL(PTR) == VARTAG_ONDISK)

    #define VARATT_IS_EXTERNAL(PTR)             VARATT_IS_1B_E(PTR)

        #define VARATT_IS_1B_E(PTR) \
            ((((varattrib_1b *) (PTR))->va_header) == 0x01)

    #define VARTAG_EXTERNAL(PTR)                VARTAG_1B_E(PTR)

        #define VARTAG_1B_E(PTR) \
            (((varattrib_1b_e *) (PTR))->va_tag)
```
*/
unsafe fn is_stale_toast(
    datum: pg::Datum,
    attr: pg::Form_pg_attribute,
) -> bool {
    use pg::vartag_external::VARTAG_ONDISK;
    let mut o: pg::Oid = 0; // Output function; not used
    let mut is_variable_length: pg::bool_ = CFALSE;
    pg::getTypeOutputInfo((*attr).atttypid, &mut o, &mut is_variable_length);
    if is_variable_length == CTRUE {
        // Cast to varlena metadata type.
        let v = datum as *const pg::varattrib_1b_e;
        if (*v).va_header != 0x01 {
            return false;
        }
        return (*v).va_tag == (VARTAG_ONDISK as u8);
    }
    return false;
}


// Symbols Postgres needs to find.

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn _PG_init() {}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn _PG_output_plugin_init(
    cb: *mut pg::OutputPluginCallbacks,
) {
    init(cb);
}


// Miscellaneous.

const CTRUE: pg::bool_ = 1;
const CFALSE: pg::bool_ = 0;

pub unsafe fn elog(file: &str, line: u32, function: &str, msg: &str) {
    let level = 15; // The LOG level of logging is normally server-only
    pg::elog_start(
        CString::new(file).unwrap().as_ptr(),
        line as ::std::os::raw::c_int,
        CString::new(function).unwrap().as_ptr(),
    );
    pg::elog_finish(
        level,
        CString::new("%s").unwrap().as_ptr(),
        CString::new(msg).unwrap().as_ptr(),
    );
}

pub unsafe fn fmt_name(name: pg::NameData) -> String {
    let cstr = CStr::from_ptr(name.data.as_ptr());
    format!("{:?}", cstr.to_owned())
}
