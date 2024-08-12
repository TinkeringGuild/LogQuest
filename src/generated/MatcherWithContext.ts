// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

/**
 * The key difference between MatcherWithContext and Matcher is that some
 * MatcherWithContext variants store a String instead of a pre-compiled Regex.
 * This is because a WatchUntilFilterMatches effect might back-reference captures
 * from an earlier Regex, whose captured values must be interpolated escaped into
 * a JIT-compiled Regex that matches on those earlier values. One consequence of
 * this is that any parse error for a Regex String would only appear later when
 * creating a Timer, so these patterns should be validated at creation-time so
 * that the values stored in the Strings are guaranteed to be error-free.
 */
export type MatcherWithContext = { "WholeLine": string } | { "PartialLine": string } | { "Pattern": string } | { "GINA": string };