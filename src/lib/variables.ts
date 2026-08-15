/**
 * Variable Replacement Utilities
 * Handles {{variableName}} placeholder replacement in strings
 */

/**
 * Replaces {{variableName}} placeholders with values from the variables object.
 * @param text - The text containing placeholders
 * @param variables - Record of variable names to their values
 * @returns Text with all placeholders replaced
 */
export function replaceVariables(
  text: string,
  variables: Record<string, string>
): string {
  return text.replace(/\{\{([^}]+)\}\}/g, (match, varName) => {
    return variables[varName] !== undefined ? variables[varName] : match;
  });
}

/**
 * Extracts all variable names from {{variableName}} placeholders.
 * @param text - The text containing placeholders
 * @returns Array of unique variable names found
 */
export function extractVariables(text: string): string[] {
  const matches = text.match(/\{\{([^}]+)\}\}/g);
  if (!matches) return [];

  const varNames = matches.map((match) => match.slice(2, -2));
  return [...new Set(varNames)];
}

/**
 * Checks if text contains any variable placeholders.
 * @param text - The text to check
 * @returns True if text contains {{variableName}} placeholders
 */
export function hasVariables(text: string): boolean {
  return /\{\{([^}]+)\}\}/.test(text);
}
