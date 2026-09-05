function at<Value>(values: readonly Value[], index: number): Value {
  const value = values[index];
  if (value === undefined) {
    throw new RangeError(`Missing test element at index ${index}`);
  }
  return value;
}

export { at };
