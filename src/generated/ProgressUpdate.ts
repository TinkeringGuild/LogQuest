// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

export type ProgressUpdate =
  | 'Started'
  | { Message: { text: string; seq: number } }
  | { Finished: { text: string; seq: number } };
