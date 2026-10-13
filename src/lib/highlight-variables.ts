function escapeHtml(text: string): string {
  return text.replace(/[&<>"']/g, char => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
  })[char]!)
}

export function highlightVariables(text: string, variables: Record<string, string>): string {
  let html = ''
  let offset = 0
  for (const match of text.matchAll(/\{\{([^}]+)\}\}/g)) {
    const index = match.index!
    html += escapeHtml(text.slice(offset, index))
    const name = match[1].trim()
    const found = name.length > 0 && Object.prototype.hasOwnProperty.call(variables, name)
    html += `{{<span class="${found ? 'var-found' : 'var-not-found'}" data-varname="${escapeHtml(name)}">${escapeHtml(match[1])}</span>}}`
    offset = index + match[0].length
  }
  return html + escapeHtml(text.slice(offset))
}
