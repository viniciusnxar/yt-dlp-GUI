// Espelha os structs Rust (serde snake_case).
export type Choice = { value: string; label: string };
export type PresetChoice = { id: string; label: string; args: string[] };

export type OptKind =
  | { type: "toggle" }
  | { type: "select"; choices: Choice[] }
  | { type: "text"; placeholder: string }
  | { type: "number"; min: number; max: number }
  | { type: "path" }
  | { type: "preset"; choices: PresetChoice[] };

export type OptionDef = {
  id: string;
  flag: string;
  label: string;
  help: string;
  group: string;
  kind: OptKind;
  default: unknown | null;
};

export type Selected = Record<string, unknown>;

export type Status = "queued" | "running" | "done" | "error" | "cancelled";

export type JobInfo = {
  id: string;
  url: string;
  title: string | null;
  status: Status;
  percent: number;
  speed: string;
  eta: string;
  filepath: string | null;
  error: string | null;
};
