var o = { inner: { value: 9, get() { return this.value; } } }; o.inner.get();
