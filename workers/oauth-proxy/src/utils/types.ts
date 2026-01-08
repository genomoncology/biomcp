/**
 * Converts all properties of a type to strings while preserving optionality.
 * Useful for modeling raw environment variables where all values are strings at runtime.
 * @example
 * AlwaysStrings<{
 *   VAR1: string;
 *   VAR2: number;
 *   VAR3?: string[];
 * }>
 * // Result:
 * // {
 * //   VAR1: string;
 * //   VAR2: string;
 * //   VAR3?: string;
 * // }
 */
export type AlwaysStrings<T> = {
  [K in keyof T]: T[K] extends string ? string : T[K] extends undefined ? string | undefined : string;
};

/**
 * Expands type intersections for better IDE hover display.
 * Instead of showing `A & B`, shows the merged object with all properties.
 * @example
 * type A = { a: number };
 * type B = { b: string };
 * type Result = Prettify<A & B>;
 * // Hover shows: { a: number; b: string } instead of A & B
 */
export type Prettify<T> = {
  [K in keyof T]: T[K];
} & {};
