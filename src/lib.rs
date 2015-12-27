extern crate libc;

#[allow(dead_code,
        non_snake_case,
        non_camel_case_types,
        non_upper_case_globals)]
pub mod libpq;


/// Symbols Postgres needs to find.

#[no_mangle]
pub unsafe extern fn _PG_init() { }

#[no_mangle]
pub unsafe extern fn
    _PG_output_plugin_init(cb: *mut libpq::OutputPluginCallbacks) { init(cb); }


/// Implementation of initialization and callbacks.

pub unsafe extern fn init(cb: *mut libpq::OutputPluginCallbacks) {
    (*cb).startup_cb = Some(startup);
    (*cb).begin_cb = Some(begin);
    (*cb).change_cb = Some(change);
    (*cb).commit_cb = Some(commit);
    (*cb).shutdown_cb = Some(shutdown);
}
/*
pub type LogicalOutputPluginInit =
    ::std::option::Option<extern "C" fn(cb: *mut Struct_OutputPluginCallbacks)
                              -> ()>;
*/

extern fn startup(ctx: *mut libpq::Struct_LogicalDecodingContext,
                  options: *mut libpq::OutputPluginOptions,
                  is_init: libpq::_bool) {

}
/*
pub type LogicalDecodeStartupCB =
    ::std::option::Option<extern "C" fn(ctx:
                                            *mut Struct_LogicalDecodingContext,
                                        options: *mut OutputPluginOptions,
                                        is_init: _bool) -> ()>;
 */

extern fn begin(ctx: *mut libpq::Struct_LogicalDecodingContext,
                txn: *mut libpq::ReorderBufferTXN) {

}
/*
pub type LogicalDecodeBeginCB =
    ::std::option::Option<extern "C" fn(arg1:
                                            *mut Struct_LogicalDecodingContext,
                                        txn: *mut ReorderBufferTXN) -> ()>;
 */

extern fn change(ctx: *mut libpq::Struct_LogicalDecodingContext,
                 txn: *mut libpq::ReorderBufferTXN,
                 relation: libpq::Relation,
                 change: *mut libpq::ReorderBufferChange) {

}
/*
pub type LogicalDecodeChangeCB =
    ::std::option::Option<extern "C" fn(arg1:
                                            *mut Struct_LogicalDecodingContext,
                                        txn: *mut ReorderBufferTXN,
                                        relation: Relation,
                                        change: *mut ReorderBufferChange)
                              -> ()>;
 */

extern fn commit(ctx: *mut libpq::Struct_LogicalDecodingContext,
                 txn: *mut libpq::ReorderBufferTXN,
                 lsn: libpq::XLogRecPtr) {

}
/*
pub type LogicalDecodeCommitCB =
    ::std::option::Option<extern "C" fn(arg1:
                                            *mut Struct_LogicalDecodingContext,
                                        txn: *mut ReorderBufferTXN,
                                        commit_lsn: XLogRecPtr) -> ()>;
 */

extern fn shutdown(ctx: *mut libpq::Struct_LogicalDecodingContext) {

}
/*
pub type LogicalDecodeShutdownCB =
    ::std::option::Option<extern "C" fn(arg1:
                                            *mut Struct_LogicalDecodingContext)
                              -> ()>;

 */
