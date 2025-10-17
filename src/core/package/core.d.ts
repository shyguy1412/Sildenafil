declare module "@core"{
  type Foo = {
    bar: string
    baz: number
  };
  type FooEvent = {
    foo: Foo
  };
  type BarEvent = {
    bar: number
    nested: Foo
  };
  function createFoo(): Foo;
  function triggerFooEvent(): number;
  const on: <T extends keyof Events>(event:T, callback:(...args:Events[T]) => void) => void;
  type Events = {  
    foo: FooEvent
    bar: BarEvent
  };
}