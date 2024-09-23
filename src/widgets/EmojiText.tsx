import React from 'react';

/**
 * Determines if a character is an emoji based on its Unicode code point.
 */
const isEmoji = (char: string): boolean => {
  const codePoint = char.codePointAt(0);

  if (!codePoint) return false;

  // Combine contiguous emoji ranges
  return (
    (codePoint >= 0x1f300 && codePoint <= 0x1f5ff) || // Misc symbols and pictographs
    (codePoint >= 0x1f600 && codePoint <= 0x1f64f) || // Emoticons
    (codePoint >= 0x1f680 && codePoint <= 0x1f6ff) || // Transport and map symbols
    (codePoint >= 0x1f700 && codePoint <= 0x1faff) || // Alchemical, Geometric Shapes Extended, Chess, etc.
    (codePoint >= 0x2600 && codePoint <= 0x27bf) // Miscellaneous symbols & Dingbats (e.g., ☀, ✈)
  );
};

/**
 * Uses reduce to combine non-emoji characters and wrap emojis.
 */
const wrapEmojis = (input: string): React.ReactNode[] => {
  const characters = Array.from(input); // Split string by Unicode characters

  return characters.reduce<React.ReactNode[]>((acc, char) => {
    if (isEmoji(char)) {
      acc.push(
        <span key={`${acc.length}-${char}`} className="emoji">
          {char}
        </span>
      );
    } else {
      // If the last entry is a string, concatenate it with the current character
      if (acc.length > 0 && typeof acc[acc.length - 1] === 'string') {
        acc[acc.length - 1] += char;
      } else {
        // Start a new string for non-emoji characters
        acc.push(char);
      }
    }

    return acc;
  }, []);
};

/**
 * React component that displays the input text with emojis wrapped in a span.
 */
const EmojiText: React.FC<{ text: string }> = ({ text }) => {
  return <>{wrapEmojis(text)}</>;
};

export default EmojiText;
