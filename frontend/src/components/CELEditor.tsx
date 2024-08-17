import { useMonaco, Editor } from "@monaco-editor/react";
import { useEffect } from "react";

export default function CELEditor({cel, onChange}: {cel?: string, onChange?: (newCel: string|undefined) => unknown}) {
  const monaco = useMonaco();

  useEffect(() => {
    if (monaco == null) {
      return;
    }
    monaco.languages.register({ id: "cel" });

    // Register a tokens provider for the language
    monaco.languages.setMonarchTokensProvider("cel", {
      keywords: ["true", "false", "null"],

      operators: [
        "&&",
        "||",
        "<=",
        "<",
        ">=",
        ">",
        "==",
        "!=",
        "in",
        "+",
        "-",
        "*",
        "/",
        "%",
        "!",
      ],

      symbols: /[=><!~?:&|+\-*/^%]+/,

      escapes:
        /\\(?:[bfnrt"\\'\\]|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8}|x[0-9A-Fa-f]{2}|0[0-8]{3})/,

      tokenizer: {
        root: [
          // Reserved identifiers (illegal in CEL)
          [
            /(\b(as|break|const|continue|else|for|function|if|import|let|loop|package|namespace|return|var|void|while)\b)/,
            "invalid",
          ],

          // Function calls
          [/[a-zA-Z_]\w*(?=\()/, "variable.function"],

          // Object construction
          [
            /[a-zA-Z_]\w*\s*(\{)/,
            [
              "variable.object",
              {
                token: "punctuation.definition.object.begin",
                bracket: "@open",
              },
            ],
          ],

          // Operators
          [
            /@symbols/,
            { cases: { "@operators": "keyword.operator", "@default": "" } },
          ],

          // Numbers
          [/\d*\.\d+([eE][-+]?\d+)?/, "number.float"],
          [/\d+[eE][-+]?\d+/, "number.float"],
          [/\d+|0x[0-9a-fA-F]+[uU]?/, "number"],

          // Strings (single-line)
          [
            /r?("|')/,
            { token: "string.quote", bracket: "@open", next: "@string" },
          ],
          // Strings (multi-line)
          [
            /r?("""|''')/,
            { token: "string.quote", bracket: "@open", next: "@multiString" },
          ],

          // Lists
          [
            /\[/,
            {
              token: "punctuation.definition.list.begin",
              bracket: "@open",
              next: "@list",
            },
          ],

          // Maps
          [
            /\{/,
            {
              token: "punctuation.definition.map.begin",
              bracket: "@open",
              next: "@map",
            },
          ],

          // Parentheses
          [
            /\(/,
            {
              token: "punctuation.parenthesis.begin",
              bracket: "@open",
              next: "@paren",
            },
          ],

          // Comments
          [/\/\/.*$/, "comment"],
        ],

        string: [
          [/[^\\'"]+/, "string"],
          [/@escapes/, "string.escape"],
          [/\\./, "string.escape.invalid"],
          [/['"]/, { token: "string.quote", bracket: "@close", next: "@pop" }],
        ],

        multiString: [
          [/[^\\'"]+/, "string"],
          [/@escapes/, "string.escape"],
          [/\\./, "string.escape.invalid"],
          [
            /("""|''')/,
            { token: "string.quote", bracket: "@close", next: "@pop" },
          ],
        ],

        list: [
          { include: "root" },
          [/,/, "punctuation.separator.list"],
          [
            /\]/,
            {
              token: "punctuation.definition.list.end",
              bracket: "@close",
              next: "@pop",
            },
          ],
        ],

        map: [
          { include: "root" },
          [/,/, "punctuation.separator.map"],
          [/:/, "punctuation.separator.map"],
          [
            /\}/,
            {
              token: "punctuation.definition.map.end",
              bracket: "@close",
              next: "@pop",
            },
          ],
        ],

        paren: [
          { include: "root" },
          [
            /\)/,
            {
              token: "punctuation.parenthesis.end",
              bracket: "@close",
              next: "@pop",
            },
          ],
        ],
      },
    });
  }, [monaco]);

  return (
    <Editor value={cel}
      language="cel"
      onChange={onChange}
      className="h-[8rem] border"
      onMount={(editor) => {
        editor.updateOptions({
          fontSize: 14,
          theme: "vs-light",
          glyphMargin: false,
          folding: false,
          lineNumbers: "on",
          lineDecorationsWidth: 2,
          lineNumbersMinChars: 2,
          minimap: { enabled: false },
          overviewRulerLanes: 0,
          scrollbar: {
            vertical: "visible",
            horizontal: "hidden",
            handleMouseWheel: true,
          },
          wordWrap: "on",
        });
      }}
    />
  );
}
