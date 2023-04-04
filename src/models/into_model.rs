pub(crate) trait IntoModel<M: Sized> {
    fn model(&self) -> crate::routes::Result<M>;
}