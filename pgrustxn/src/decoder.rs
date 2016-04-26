#[cfg(pg94)]
pub use pg94 as pg;

//#[cfg(pg95)]
pub use pg95 as pg;

use std::ffi::CString;

use std::mem::size_of;

pub fn change(ctx: *mut pg::Struct_LogicalDecodingContext,
              _txn: *mut pg::ReorderBufferTXN,
              relation: pg::Relation,
              change: *mut pg::ReorderBufferChange) {
    unsafe {
        let relid = (*relation).rd_id;
        let last_relid: *mut pg::Oid =
            (*ctx).output_plugin_private as *mut pg::Oid;
        if *last_relid != relid {
            pg::OutputPluginPrepareWrite(ctx, CFALSE);
            append_schema(relation, (*ctx).out);
            pg::OutputPluginWrite(ctx, CFALSE);
            *last_relid = relid;
        }
    }
    unsafe {
        pg::OutputPluginPrepareWrite(ctx, CTRUE);
        append_change(relation, change, (*ctx).out);
        pg::OutputPluginWrite(ctx, CTRUE);
    }
}

unsafe fn append_change(relation: pg::Relation,
                        change: *mut pg::ReorderBufferChange,
                        out: pg::StringInfo) {
    let tuple_desc = (*relation).rd_att;
    let tuples = (*change).data.tp();
    let tuple_new = (*tuples).newtuple;
    let tuple_old = (*tuples).oldtuple;
    let token = match (*change).action {
        pg::Enum_ReorderBufferChangeType::REORDER_BUFFER_CHANGE_INSERT => "insert",
        pg::Enum_ReorderBufferChangeType::REORDER_BUFFER_CHANGE_UPDATE => "update",
        pg::Enum_ReorderBufferChangeType::REORDER_BUFFER_CHANGE_DELETE => "delete",
        _ => panic!("Unrecognized change action!"),
    };
    append("{ ", out);
    append("\"", out);
    append(token, out);
    append("\": ", out);
    append_tuple_buf_as_json(tuple_new, tuple_desc, out);
    if !tuple_old.is_null() {
        append(", \"@\": ", out);
        append_tuple_buf_as_json(tuple_old, tuple_desc, out);
    }
    append(" }", out);
}

unsafe fn append_tuple_buf_as_json(data: *mut pg::ReorderBufferTupleBuf,
                                   desc: pg::TupleDesc,
                                   out: pg::StringInfo) {
    if !data.is_null() {
        let heap_tuple = &mut (*data).tuple;
        let datum = pg::heap_copy_tuple_as_datum(heap_tuple, desc);
        let empty_oid: pg::Oid = 0;
        let json = pg::DirectFunctionCall1Coll(Some(row_to_json),
                                               empty_oid,
                                               datum);
        let ptr = json as *const pg::Struct_varlena;
        let text = pg::text_to_cstring(ptr);
        pg::appendStringInfoString(out, text);
    } else {
        append("{}", out);
    }
}

unsafe fn append<T: Into<Vec<u8>>>(t: T, out: pg::StringInfo) {
    pg::appendStringInfoString(out, CString::new(t).unwrap().as_ptr());
}

unsafe fn append_schema(relation: pg::Relation, out: pg::StringInfo) {
    let relid = (*relation).rd_id;
    let tupdesc = (*relation).rd_att;
    let name = pg::get_rel_name(relid);
    let ns = pg::get_namespace_name(pg::get_rel_namespace(relid));
    let qualified_name = pg::quote_qualified_identifier(ns, name);
    append("{ \"table\": ", out);
    append("\"", out);
    pg::appendStringInfoString(out, qualified_name);
    append("\"", out);
    append(",", out);
    append(" \"schema\": ", out);
    append("[", out);
    let fmt = CString::new("{\"%s\":\"%s\"}").unwrap();
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
            append(",", out);
        } else {
            first = false;
        }
        pg::appendStringInfo(out, fmt.as_ptr(), col, typ);
    }
    append("]", out);
    append(" }", out);
}

extern "C" fn row_to_json(fcinfo: pg::FunctionCallInfo) -> pg::Datum {
    // We wrap the unsafe call to make it safe, so that it can be passed as
    // a function pointer to DirectFunctionCall1Coll(). This is a spurious
    // artifact of the generated binding.
    unsafe { pg::row_to_json(fcinfo) }
}

const CTRUE: pg::_bool = 1;
const CFALSE: pg::_bool = 0;

// Miscellaneous.
