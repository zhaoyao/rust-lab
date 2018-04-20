extern crate futures;

use futures::prelude::*;
use futures::future::*;
use std::io;


fn main() {

    let f1 = futures::lazy(|| ok::<u32, u32>(1));
    test_send( f1 );

    //let f1: Box<Future<Item=u32, Error=u32> + Send> = Box::new(futures::lazy(|| ok::<u32, u32>(1)));
    //test_send( (f1) );

    let f2 = fn_return_box_future(1, 1);
    test_send( f2 );

    // 返回的 Box<Future> 需要显示添加 Send annotation
    //let f3 = fn_return_box_future_no_send2(1, 1);
    //test_send( f3 );

    test_send(
        futures::lazy(|| ok::<u32, u32>(1)).and_then(|_| {
            // future result
            //result(Ok(1))

            // compiler error: not safe
            //fn_return_box_future_no_send(1, 1)
            //fn_return_box_future_no_send2(1, 1)

            // works
            fn_return_impl_future(1, 1)

            // chain 中任意一步存在 !Send, 都会导致整个 chain !Send
            //fn_return_box_future_no_send(1, 1)
            //fn_return_impl_future(1, 1)
                //.and_then( |_| fn_return_box_future(1, 1) )
        }));

}

fn test_send<T, E, F>(f: F) -> Box<Future<Item = T, Error = E> >
where
    F: Future<Item = T, Error = E> + Send + 'static,
{
    Box::new(f)
}



fn fn_return_box_future<T, E>(_t: T, _e: E) -> Box<Future<Item = T, Error = E> + Send> {
    panic!("")
}


fn fn_return_box_future_no_send<T, E>(_t: T, _e: E) -> Box<Future<Item = T, Error = E>> {
    panic!("")
}

fn fn_return_box_future_no_send2<T, E>(_t: T, _e: E) -> Box<Future<Item = T, Error = E>>
    where T: Send {
    panic!("")
}

fn fn_return_impl_future(_t: u32, _e: u32) -> impl Future<Item=u32, Error=u32> {
    result(Ok(1))
}
