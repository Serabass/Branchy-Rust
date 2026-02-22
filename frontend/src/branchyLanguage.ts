import type { languages } from 'monaco-editor'

export const branchyLanguage: languages.IMonarchLanguage = {
  defaultToken: '',
  tokenPostfix: '.branchy',

  keywords: ['include'],
  operators: ['=', '+', '*', '..', '...', ';', ',', ':', ':?'],
  brackets: [
    { open: '[', close: ']', token: 'delimiter.square' },
    { open: '{', close: '}', token: 'delimiter.curly' },
    { open: '(', close: ')', token: 'delimiter.paren' },
    { open: '<', close: '>', token: 'delimiter.angle' },
  ],

  tokenizer: {
    root: [
      { include: '@whitespace' },
      [/[@!~]/, 'keyword.control'],
      [/include\b/, 'keyword'],
      [/[\[\]\{\}\(\)<>\|\;,=]/, 'delimiter'],
      [/\.\.\.?/, 'operator'],
      [/[+\*]/, 'operator'],
      [/:(\?)?[a-zA-Z_][a-zA-Z0-9_]*/, 'variable.parameter'],
      [/"([^"\\]|\\.)*"/, 'string.double'],
      [/'([^'\\]|\\.)*'/, 'string.single'],
      [/[a-zA-Z_][a-zA-Z0-9_]*/, 'identifier'],
      [/[0-9]+/, 'number'],
    ],
    whitespace: [[/\s+/, 'white']],
  },
}

export const branchyLanguageId = 'branchy'
