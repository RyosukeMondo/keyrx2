import { Editor } from '@monaco-editor/react';

/**
 * Simple code editor without WASM validation
 * Lightweight alternative to MonacoEditor for pages that don't need validation
 */
interface SimpleCodeEditorProps {
  value: string;
  onChange?: (value: string) => void;
  readOnly?: boolean;
  height?: string;
  language?: string;
}

export function SimpleCodeEditor({
  value,
  onChange,
  readOnly = false,
  height = '600px',
  language = 'javascript',
}: SimpleCodeEditorProps) {
  const handleChange = (newValue: string | undefined) => {
    if (onChange && newValue !== undefined) {
      onChange(newValue);
    }
  };

  return (
    <Editor
      height={height}
      language={language}
      theme="vs-dark"
      value={value}
      onChange={handleChange}
      options={{
        readOnly,
        minimap: { enabled: false },
        fontSize: 14,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        automaticLayout: true,
        wordWrap: 'on',
        tabSize: 2,
        insertSpaces: true,
      }}
    />
  );
}
