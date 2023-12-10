export interface ObjectVariable {
  name: string;
  type: string;
};

export type SelectedVariables = {
  variable: ObjectVariable;
  qualifier: string;
  bound: boolean;
}[];


export type CountConstraint = {min: number, max: number};