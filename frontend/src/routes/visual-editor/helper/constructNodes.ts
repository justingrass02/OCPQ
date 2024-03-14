import type { EventTypeQualifiers } from "@/types/ocel";
import type {
  DiscoveredCountConstraint,
  DiscoveredEFConstraint,
  DiscoveredORConstraint,
  EventTypeLinkData,
  EventTypeNodeData,
  GateLinkData,
  GateNodeData,
  ObjectVariable,
  ViolationsPerNodes,
} from "./types";
import { EVENT_TYPE_LINK_TYPE, GATE_LINK_TYPE, GATE_NODE_TYPE } from "./const";
import {type Edge, MarkerType, type ReactFlowJsonObject } from "reactflow";

export function constructDiscoveredCountConstraint(
  c: DiscoveredCountConstraint,
  qualifiers: EventTypeQualifiers,
) {
  const countStr =
    c.countConstraint.min === c.countConstraint.max
      ? `=${c.countConstraint.min}`
      : c.countConstraint.min === 0
      ? `<=${c.countConstraint.max}`
      : `${c.countConstraint.min} - ${c.countConstraint.max}`;
  const name = `${countStr} "${
    c.eventType.type === "exactly" ? c.eventType.value : "..."
  }" per ${c.objectType}`;
  const varName = c.objectType.substring(0, 2) + "_0";
  const variable = {
    name: varName,
    type: c.objectType,
    initiallyBound: true,
  };
  const constraintFlow: {
    flowJson: ReactFlowJsonObject<EventTypeNodeData, EventTypeLinkData>;
    violations?: ViolationsPerNodes;
    objectVariables?: ObjectVariable[];
  } = {
    violations: undefined,
    // TODO: Add object variable
    objectVariables: [variable],
    flowJson: {
      nodes: [
        {
          id: Date.now() + "auto" + Math.random(),
          type: "eventType",
          position: { x: 150, y: 150 },
          data: {
            eventType: c.eventType,
            eventTypeQualifier:
              qualifiers[
                c.eventType.type === "exactly" ? c.eventType.value : ""
              ],
            countConstraint: c.countConstraint,
            selectedVariables: [
              {
                qualifier: undefined,
                variable,
                bound: false,
              },
            ],
          },
        },
      ],
      edges: [],
      viewport: { x: 0, y: 0, zoom: 2.0 },
    },
  };
  return {
    constraint: constraintFlow,
    name,
    description: "Automatically Discovered Constraint",
  };
}

export function constructDiscoveredEFConstraint(
  c: DiscoveredEFConstraint,
  qualifiers: EventTypeQualifiers,
) {
  // updatedConstraints.push({
  //   name,
  //   description:
  //     "Automatically Discovered Constraint",
  // });

  const varNames: string[] = [];
  for (const ot of c.objectTypes) {
    const shortOt = ot.substring(0, 2);
    let i = 0;
    let varName = shortOt + "_" + i;
    while (varNames.includes(varName)) {
      i++;
      varName = shortOt + "_" + i;
    }
    varNames.push(varName);
  }
  const variables = varNames.map((varName, i) => ({
    name: varName,
    type: c.objectTypes[i],
    initiallyBound: i === 0,
  }));
  const ids = [
    Date.now() + "auto-ef-" + Math.random(),
    Date.now() + "auto-ef2-" + Math.random(),
  ] as const;
  const name = `${c.fromEventType} -> ${c.toEventType} for ${c.objectTypes.join(
    ", ",
  )}`;

  const X: {
    flowJson: ReactFlowJsonObject<EventTypeNodeData, EventTypeLinkData>;
    violations?: ViolationsPerNodes;
    objectVariables?: ObjectVariable[];
  } = {
    violations: undefined,
    objectVariables: variables,
    flowJson: {
      nodes: [
        {
          id: ids[0],
          type: "eventType",
          position: { x: 200, y: 100 },
          data: {
            eventType: {
              type: "exactly",
              value: c.fromEventType,
            },
            eventTypeQualifier: qualifiers[c.fromEventType],
            countConstraint: {
              min: 0,
              max: Infinity,
            },
            selectedVariables: variables.map((variable, i) => ({
              qualifier: undefined,
              variable,
              bound: i > 0,
            })),
            hideViolations: true,
          },
        },
        {
          id: ids[1],
          type: "eventType",
          position: { x: 200, y: 350 },
          data: {
            eventType: {
              type: "exactly",
              value: c.toEventType,
            },
            eventTypeQualifier: qualifiers[c.toEventType],
            countConstraint: {
              min: 1,
              max: Infinity,
            },

            selectedVariables: variables.map((variable) => ({
              qualifier: undefined,
              variable,
              bound: false,
            })),
            hideViolations: false,
          },
        },
      ],
      edges: [
        {
          type: EVENT_TYPE_LINK_TYPE,
          source: ids[0],
          target: ids[1],
          sourceHandle: ids[0] + "-source",
          targetHandle: ids[1] + "-target",
          id: Date.now() + "link-ef-" + Math.random(),
          markerEnd: {
            type: MarkerType.ArrowClosed,
            width: 15,
            height: 12,
            color: "#969696",
          },
          style: {
            strokeWidth: 2,
            stroke: "#969696",
          },
          data: {
            color: "#969696",
            constraintType: "response",
            timeConstraint: c.secondsRange,
          },
        },
      ],
      viewport: { x: 0, y: 0, zoom: 1.5 },
    },
  };
  return {
    constraint: X,
    name,
    description: "Automatically Discovered Constraint",
  };
}

export function constructDiscoveredORConstraint(
  c: DiscoveredORConstraint,
  qualifiers: EventTypeQualifiers,
) {
  if ("EfOrCount" in c) {
    const [efConstr, countConstr] = c.EfOrCount;
    const efFlow = constructDiscoveredEFConstraint(efConstr, qualifiers);
    const countFlow = constructDiscoveredCountConstraint(
      countConstr,
      qualifiers,
    );
    countFlow.constraint.flowJson.nodes[0].position.x += 100;
    const name = "OR " + efFlow.name + " / " + countFlow.name;
    const orNodeID = Math.random() + "auto_OR" + Date.now();
    const newEdges: Edge<GateLinkData>[] = [
      {
        id: Math.random() + "OR_AUTO_LINK" + Date.now(),
        type: GATE_LINK_TYPE,
        source: orNodeID,
        sourceHandle: orNodeID + "-left-source",
        target: efFlow.constraint.flowJson.nodes[0].id,
        targetHandle: efFlow.constraint.flowJson.nodes[0].id + "-target",
        markerEnd: {
          type: MarkerType.ArrowClosed,
          width: 15,
          height: 12,
          // color,
        },
        style: {
          strokeWidth: 2,
          // stroke: color,
        },
      },
      {
        id: Math.random() + "OR_AUTO_LINK" + Date.now(),
        type: GATE_LINK_TYPE,
        source: orNodeID,
        sourceHandle: orNodeID + "-right-source",
        target: countFlow.constraint.flowJson.nodes[0].id,
        targetHandle: countFlow.constraint.flowJson.nodes[0].id + "-target",
        markerEnd: {
          type: MarkerType.ArrowClosed,
          width: 15,
          height: 12,
          // color,
        },
        style: {
          strokeWidth: 2,
          // stroke: color,
        },
      },
    ];
    const constraint: {
      flowJson: ReactFlowJsonObject<
        EventTypeNodeData | GateNodeData,
        EventTypeLinkData | GateLinkData
      >;
      violations?: ViolationsPerNodes;
      objectVariables?: ObjectVariable[];
    } = {
      violations: undefined,
      objectVariables: efFlow.constraint.objectVariables,
      flowJson: {
        nodes: [
          {
            type: GATE_NODE_TYPE,
            id: orNodeID,
            position: { x: 50, y: -50 },
            data: { type: "or" },
          },
          ...efFlow.constraint.flowJson.nodes,
          ...countFlow.constraint.flowJson.nodes,
        ],
        edges: [...newEdges, ...efFlow.constraint.flowJson.edges],
        viewport: { x: 0, y: -50, zoom: 1.5 },
        // nodes: [...efFlow.constraint.flowJson.nodes, ...countFlow.constraint.flowJson.nodes],
      },
    };
    return {
      constraint,
      name,
      description: "Automatically Discovered Constraint",
    };
  } else {
    console.error("NOT IMPLEMENTED (YET)");
    return undefined;
    // const [countConstr, efConstr] = c.CountOrEf;
  }
  // TODO: Multiple object types?!
  // const objectType = "EfOrCount" in c ? c.EfOrCount[0].objectTypes[0] : c.CountOrEf[1].objectTypes[0];
  // const varName = objectType.substring(0, 2) + "_0";
  // const variable = {
  //   name: varName,
  //   type: objectType,
  //   initiallyBound: true,
  // };
}
