extern crate libc;
use std::ffi::CString;
use std::mem::size_of;
extern crate pgrustxn;

#[cfg(pg94)]
pub use pgrustxn::pg94 as pg;

#[cfg(pg95)]
pub use pgrustxn::pg95 as pg;

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
    let last_relid = pg::palloc0(size_of::<pg::Oid>() as u64);
    (*ctx).output_plugin_private = last_relid;
    pgrustxn::texttools::set_output_to_textual(ctx, options, _is_init)
}

unsafe extern "C" fn begin(ctx: *mut pg::Struct_LogicalDecodingContext,
                           txn: *mut pg::ReorderBufferTXN) {
    pgrustxn::texttools::write_text1(ctx, "{ \"begin\": %u }", (*txn).xid);
}

unsafe extern "C" fn change(ctx: *mut pg::Struct_LogicalDecodingContext,
                            _txn: *mut pg::ReorderBufferTXN,
                            relation: pg::Relation,
                            change: *mut pg::ReorderBufferChange) {
    pgrustxn::decoder::change(ctx, _txn, relation, change);
}

unsafe extern "C" fn commit(ctx: *mut pg::Struct_LogicalDecodingContext,
                            txn: *mut pg::ReorderBufferTXN,
                            _lsn: pg::XLogRecPtr) {
    let t = pg::timestamptz_to_str((*txn).commit_time);
    pgrustxn::texttools::write_text2(ctx,
                                     "{ \"commit\": %u, \"t\": \"%s\" }",
                                     (*txn).xid,
                                     t);
    let last_relid: *mut pg::Oid =
        (*ctx).output_plugin_private as *mut pg::Oid;
    *last_relid = 0;
}

unsafe extern "C" fn shutdown(ctx: *mut pg::Struct_LogicalDecodingContext) {
    pg::pfree((*ctx).output_plugin_private);
}
