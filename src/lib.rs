extern crate libc;
use std::ffi::CString;
use std::mem::size_of;

extern crate rpgffi as pg;

// Implementation of initialization and callbacks.

pub unsafe extern "C" fn init(cb: *mut pg::OutputPluginCallbacks) {
    (*cb).startup_cb = Some(startup);
    (*cb).begin_cb = Some(begin);
    (*cb).change_cb = Some(change);
    (*cb).commit_cb = Some(commit);
    (*cb).shutdown_cb = Some(shutdown);
}

unsafe extern "C" fn startup(ctx: *mut pg::Struct_LogicalDecodingContext,
                             options: *mut pg::OutputPluginOptions,
                             _is_init: pg::_bool) {
    use pg::Enum_OutputPluginOutputType::*;
    let last_relid = pg::palloc0(size_of::<pg::Oid>() as u64);
    (*ctx).output_plugin_private = last_relid;
    (*options).output_type = OUTPUT_PLUGIN_TEXTUAL_OUTPUT;
}

unsafe extern "C" fn begin(ctx: *mut pg::Struct_LogicalDecodingContext,
                           txn: *mut pg::ReorderBufferTXN) {
    let s = CString::new("{ \"begin\": %u }").unwrap();
    pg::OutputPluginPrepareWrite(ctx, CTRUE);
    pg::appendStringInfo((*ctx).out, s.as_ptr(), (*txn).xid);
    pg::OutputPluginWrite(ctx, CTRUE);
}

unsafe extern "C" fn change(ctx: *mut pg::Struct_LogicalDecodingContext,
                            _txn: *mut pg::ReorderBufferTXN,
                            relation: pg::Relation,
                            change: *mut pg::ReorderBufferChange) {
    let relid = (*relation).rd_id;
    let last_relid: *mut pg::Oid =
        (*ctx).output_plugin_private as *mut pg::Oid;
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

unsafe extern "C" fn commit(ctx: *mut pg::Struct_LogicalDecodingContext,
                            txn: *mut pg::ReorderBufferTXN,
                            _lsn: pg::XLogRecPtr) {
    let s = CString::new("{ \"commit\": %u, \"t\": \"%s\" }").unwrap();
    let t = pg::timestamptz_to_str((*txn).commit_time);
    pg::OutputPluginPrepareWrite(ctx, CTRUE);
    pg::appendStringInfo((*ctx).out, s.as_ptr(), (*txn).xid, t);
    pg::OutputPluginWrite(ctx, CTRUE);
    let last_relid: *mut pg::Oid =
        (*ctx).output_plugin_private as *mut pg::Oid;
    *last_relid = 0;
}

unsafe extern "C" fn shutdown(ctx: *mut pg::Struct_LogicalDecodingContext) {
    pg::pfree((*ctx).output_plugin_private);
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
    unsafe fn add_str(self, t: *mut i8) {
        pg::appendStringInfoString(self, t);
    }
    unsafe fn add_json(self, t: *mut i8) {
        pg::escape_json(self, t);
    }
}

impl PGAppend<pg::Datum> for pg::StringInfo {
    unsafe fn add_str(self, datum: pg::Datum) {
        self.add_json(datum);
    }
    unsafe fn add_json(self, datum: pg::Datum) {
        let empty_oid: pg::Oid = 0;
        let json =
            pg::DirectFunctionCall1Coll(Some(row_to_json), empty_oid, datum);
        let ptr = json as *const pg::Struct_varlena;
        pg::appendStringInfoString(self, pg::text_to_cstring(ptr));
    }
}

unsafe fn append_change(relation: pg::Relation,
                        change: *mut pg::ReorderBufferChange,
                        out: pg::StringInfo) {
    use pg::Enum_ReorderBufferChangeType::*;
    let relid = (*relation).rd_id;
    let name = pg::get_rel_name(relid);
    let ns = pg::get_namespace_name(pg::get_rel_namespace(relid));
    let qualified_name = pg::quote_qualified_identifier(ns, name);
    let tuple_desc = (*relation).rd_att;
    let tuples = (*change).data.tp();
    let tuple_new = (*tuples).newtuple;
    let tuple_old = (*tuples).oldtuple;
    let token = match (*change).action {
        REORDER_BUFFER_CHANGE_INSERT => "insert",
        REORDER_BUFFER_CHANGE_UPDATE => "update",
        REORDER_BUFFER_CHANGE_DELETE => "delete",
        _ => panic!("Unrecognized change action!"),
    };
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

unsafe fn append_tuple_buf_as_json(data: *mut pg::ReorderBufferTupleBuf,
                                   desc: pg::TupleDesc,
                                   out: pg::StringInfo) {
    if !data.is_null() {
        let heap_tuple = &mut (*data).tuple;
        let n = (*desc).natts as usize;

        let mut datums: Vec<pg::Datum> = Vec::new();
        datums.resize(n, 0);
        let mut nulls: Vec<pg::_bool> = Vec::new();
        nulls.resize(n, CFALSE);
        pg::heap_deform_tuple(heap_tuple,
                              desc,
                              datums.as_mut_ptr(),
                              nulls.as_mut_ptr());

        for i in 0..n {
            let datum: pg::Datum = datums[i];
            if !is_toast(datum) {
                continue;
            }
            out.add_json(datum);
        }

        // TODO: Append unchanged ones.
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
        if (*attr).attisdropped == 1 || num <= 0 {
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

extern "C" fn row_to_json(fcinfo: pg::FunctionCallInfo) -> pg::Datum {
    // We wrap the unsafe call to make it safe, so that it can be passed as
    // a function pointer to DirectFunctionCall1Coll(). This is a spurious
    // artifact of the generated binding.
    unsafe { pg::row_to_json(fcinfo) }
}


fn is_toast(datum: pg::Datum) -> bool {
    false
}


// Symbols Postgres needs to find.

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn _PG_init() {}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern fn
    _PG_output_plugin_init(cb: *mut pg::OutputPluginCallbacks) {
    init(cb);
}


// Miscellaneous.

const CTRUE: pg::_bool = 1;
const CFALSE: pg::_bool = 0;
