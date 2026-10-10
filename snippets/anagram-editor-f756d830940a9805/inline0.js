
export function request_exclusive(name) {
  if (!navigator.locks) {
    return Promise.resolve(() => {});
  }
  return new Promise((granted) => {
    navigator.locks.request(name, { ifAvailable: true }, (lock) => {
      if (!lock) {
        granted(null);
        return undefined;
      }
      return new Promise((release) => granted(release));
    });
  });
}
