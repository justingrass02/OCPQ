/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable no-template-curly-in-string */
import { VisualEditorContext } from "@/routes/visual-editor/helper/VisualEditorContext";
import type { Variable } from "@/types/generated/Variable";
import { useMonaco, Editor } from "@monaco-editor/react";
import { useContext, useEffect, useMemo } from "react";

export default function CELEditor({
  cel,
  onChange,
  availableEventVars,
  availableObjectVars,
  nodeID,
}: {
  cel?: string;
  onChange?: (newCel: string | undefined) => unknown;
  availableEventVars: number[];
  availableObjectVars: number[];
  nodeID: string;
}) {
  const monaco = useMonaco();
  const varSymbols = useMemo(() => {
    return [
      ...availableObjectVars.map((ov) => "o" + (ov + 1)),
      ...availableEventVars.map((ev) => "e" + (ev + 1)),
    ];
  }, [availableEventVars, availableObjectVars]);

  const { getTypesForVariable, getVarName } = useContext(VisualEditorContext);
  const specialFunctions = [
    {
      name: "attr",
      for_type: ["object", "event"],
      insertTemplate: "attr(${1:attr_name})",
      signatureLabel: "string attr(attr_name: string)",
      parameters: [
        {
          label: "attr_name: string",
          documentation: "The name of the attribute.",
        },
      ],
      description:
        "**attr**\n\nRetrieve an object/event attribute value.\n\nExample:\n`o1.attr('price') >= 100`<br/><br/>For objects the first encountered attribute value is picked, regardless of the associated timestamp. Use `attr_at` to retrieve the attribute value of an object at a specific timestamp. ",
    },
    {
      name: "attr_at",
      for_type: ["object"],
      insertTemplate: "attr_at(${1:attr_name},${2:time})",

      signatureLabel: "string attr(attr_name: string,at_time: timestamp)",
      parameters: [
        {
          label: "attr_name: string",
          documentation: "The name of the attribute.",
        },
        {
          label: "at_time: timestamp",
          documentation:
            "The timestamp on which to retrieve the current attribute value.",
        },
      ],
      description:
        "**attr_at**\n\nRetrieve an *object* attribute value.\n\nExamples:\n`o1.attr('price',e1.time()) >= 100`,\n\n`o1.attr('price',timestamp('2024-01-01T12:30:00+00:00')) >= 50.0`<br/><br/>The latest recorded attribute value before or at the specified timestamp is selected.<br/>See `attr` for also retrieving attributes of events.",
    },
    {
      name: "time",
      for_type: ["event"],
      insertTemplate: "time()",
      description:
        "**time**\n\nRetrieve the **timestamp of an event**.\n\nExample:\n`e2.time() - e1.time() <= duration('24h')`",
    },
    {
      name: "type",
      for_type: ["object", "event"],
      insertTemplate: "type()",
      description:
        "**type**\n\nRetrieve the **type of an object/event**\n\nExample:\n`o1.type() == 'orders'`",
    },
  ];
  useEffect(() => {
    if (monaco == null) {
      return;
    }
    const dispos: { dispose: () => unknown }[] = [];
    const specialSymbols = varSymbols.map(
      (s) => [s, "constant"] as [string, string],
    );
    const standardFunctionNames = [
      { value: "timestamp" },
      { value: "string" },
      { value: "duration" },
    ];
    monaco.languages.register({ id: "cel" });

    // Register a tokens provider for the language
    monaco.languages.setMonarchTokensProvider("cel", {
      defaultToken: "invalid",
      customFunctions: specialFunctions.map((s) => s.name),
      standardFunctions: standardFunctionNames.map((s) => s.value),
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

          // Custom :)
          ...specialSymbols,
          [
            /[a-z_$][\w$]*/,
            {
              cases: {
                "@customFunctions": "keyword.function.custom",
                "@standardFunctions": "keyword.function",
                "@keywords": "constant",
                "@default": "identifier",
              },
            },
          ],

          // // Function calls
          // [/[a-zA-Z_]\w*(?=\()/, "function"],
          // ...specialFunctionNames,

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
    dispos.push(
      monaco.languages.registerCompletionItemProvider("cel", {
        provideCompletionItems: (_model, _position) => {
          console.log(varSymbols);
          return {
            suggestions: varSymbols.map((s) => ({
              label: s,
              insertText: s,
              filterText: s,
              detail: s.startsWith("o") ? "Object Variable" : "Event Variable",
              range: null as any,
              kind: monaco.languages.CompletionItemKind.Variable, // CompletionItemKind Variable
            })),
          };
        },
      }),
    );
    dispos.push(
      monaco.languages.registerSignatureHelpProvider("cel", {
        signatureHelpTriggerCharacters: ["(", ","],
        signatureHelpRetriggerCharacters: [")"],
        provideSignatureHelp: (model, pos, _token) => {
          const prev = model.getValueInRange({
            startLineNumber: pos.lineNumber,
            endLineNumber: pos.lineNumber,
            startColumn: 0,
            endColumn: pos.column,
          });
          let open = 0;
          let close = 0;
          let i = prev.length - 1;
          let paraPos = 0;
          let buf = "";

          while (i >= 0) {
            const c = prev.charAt(i);
            if (c === "(") {
              open++;
              buf = "";
              // paraPosPrev = paraPos;
              // paraPos = 0;
            } else if (c === ")") {
              close++;
              buf = "";
            } else if (c === " " || c === "," || c === ".") {
              if (c === "," && close <= open) {
                paraPos++;
              }
              if (open > close && buf.length > 0) {
                console.log("Bling Bling", buf, paraPos);
                break;
              }
              buf = "";
            } else {
              buf = c + buf;
            }
            i--;
          }
          const sf = specialFunctions.find((sf) => sf.name === buf);

          console.log("signature help", prev, sf);
          if (sf == null) {
            return {
              value: { signatures: [], activeParameter: 0, activeSignature: 0 },
              dispose: () => {},
            };
          }

          return {
            value: {
              signatures: [
                {
                  label: sf.signatureLabel ?? sf.name,
                  parameters: sf.parameters ?? [],
                },
              ],
              activeParameter: paraPos,
              activeSignature: 0,
            },
            dispose: () => {},
          };
        },
      }),
    );
    dispos.push(
      monaco.languages.registerCompletionItemProvider("cel", {
        triggerCharacters: ["."],
        provideCompletionItems: (model, position) => {
          const word =
            model.getWordAtPosition(position.delta(0, -2))?.word ?? "";
          if (varSymbols.includes(word)) {
            const wordType = word.startsWith("o") ? "object" : "event";
            return {
              suggestions: specialFunctions
                .filter((s) => s.for_type.includes(wordType))
                .map((s) => ({
                  label: s.name,
                  insertText: s.insertTemplate ?? s.name,
                  filterText: s.name,
                  detail: s.description,
                  range: null as any,
                  kind: monaco.languages.CompletionItemKind.Function, // CompletionItemKind Function
                  insertTextRules:
                    monaco.languages.CompletionItemInsertTextRule
                      .InsertAsSnippet,
                })),
            };
          } else {
            return { suggestions: [] };
          }
        },
      }),
    );

    dispos.push(
      monaco.languages.registerHoverProvider("cel", {
        provideHover: (model, position, _token) => {
          const word = model.getWordAtPosition(position)?.word ?? "";
          if (varSymbols.includes(word)) {
            const variableIndex = parseInt(word.substring(1)) - 1;
            const variable: Variable = word.startsWith("o")
              ? { Object: variableIndex }
              : { Event: variableIndex };
            const { color } = getVarName(
              variableIndex,
              "Object" in variable ? "object" : "event",
            );
            // const hintText = `<h3 style="font-size:18px!important;"><span style="color:${color};">${word}</span>: ${
            //   "Object" in variable ? "**Object**" : "Event"
            // } Variable</h3> <br/>
            // Allowed Types: ${getTypesForVariable(
            //   nodeID,
            //   variableIndex,
            //   "Object" in variable ? "object" : "event",
            // )
            //   .map((ot) => ot.name)
            //   .join(", ")}`;
            const hintText = `<span style="color:${color};">**${word}**</span>: ${
              "Object" in variable ? "**Object" : "**Event"
            } Variable**\n\nAllowed Types: ${getTypesForVariable(
              nodeID,
              variableIndex,
              "Object" in variable ? "object" : "event",
            )
              .map((ot) => ot.name)
              .join(", ")}`;
            return {
              contents: [{ value: hintText, supportHtml: true }],
            };
          } else if (specialFunctions.find((s) => s.name === word) != null) {
            const sf = specialFunctions.find((s) => s.name === word)!;
            console.log({ sf });
            return { contents: [{ value: sf.description, supportHtml: true }] };
          }
        },
      }),
    );
    return () => {
      dispos.forEach((d) => d.dispose());
    };
  }, [monaco]);

  return (
    <Editor
      value={cel}
      language="cel"
      onChange={onChange}
      className="min-h-[10rem] border rounded"
      onMount={(editor) => {
        editor.updateOptions({
          fontSize: 16,
          theme: "vs-light",
          glyphMargin: false,
          folding: true,
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
