//
// Copyright 2020 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

use futures::try_join;
use neon::prelude::*;
use signal_neon_futures::*;

struct NameStore {
    context: JsAsyncContext,
    key: JsAsyncContextKey<JsObject>,
}

impl NameStore {
    fn new<'a>(
        cx: &mut FunctionContext<'a>,
        store: Handle<'a, JsObject>,
        context: JsAsyncContext,
    ) -> Self {
        let key = context.register_context_data(cx, store);
        Self { context, key }
    }

    async fn get_name(&self) -> Result<String, String> {
        self.context
            .await_promise(|cx| {
                let store_object = self.context.get_context_data(cx, self.key);
                let op = store_object
                    .get(cx, "getName")?
                    .downcast_or_throw::<JsFunction, _>(cx)?;
                op.call(cx, store_object, std::iter::empty::<Handle<JsValue>>())?
                    .downcast_or_throw(cx)
            })
            .then(|cx, result| match result {
                Ok(value) => match value.downcast::<JsString, _>(cx) {
                    Ok(s) => Ok(s.value(cx)),
                    Err(_) => Err("name must be a string".into()),
                },
                Err(error) => Err(error
                    .to_string(cx)
                    .expect("can convert to string")
                    .value(cx)),
            })
            .await
    }
}

async fn double_name_from_store_impl(store: NameStore) -> Result<String, String> {
    Ok(format!(
        "{0} {1}",
        store.get_name().await?,
        store.get_name().await?
    ))
}

// function doubleNameFromStore(store: { getName: () => Promise<string> }): Promise<string>
pub fn double_name_from_store(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_store = cx.argument(0)?;

    promise(&mut cx, |cx, future_context| {
        let store = NameStore::new(cx, js_store, future_context);
        async move {
            let result = double_name_from_store_impl(store).await;
            fulfill_promise(move |cx| match result {
                Ok(doubled) => Ok(cx.string(doubled)),
                Err(message) => cx.throw_error(format!("rejected: {}", message)),
            })
        }
    })
}

async fn double_name_from_store_using_join_impl(store: NameStore) -> Result<String, String> {
    let names = try_join!(store.get_name(), store.get_name())?;
    Ok(format!("{0} {1}", names.0, names.1))
}

// function doubleNameFromStoreUsingJoin(store: { getName: () => Promise<string> }): Promise<string>
pub fn double_name_from_store_using_join(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_store = cx.argument(0)?;

    promise(&mut cx, |cx, future_context| {
        let store = NameStore::new(cx, js_store, future_context);
        async move {
            let result = double_name_from_store_using_join_impl(store).await;
            fulfill_promise(move |cx| match result {
                Ok(doubled) => Ok(cx.string(doubled)),
                Err(message) => cx.throw_error(format!("rejected: {}", message)),
            })
        }
    })
}
