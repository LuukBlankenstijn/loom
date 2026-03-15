import { create } from "@bufbuild/protobuf";
import {
  CustomCommandSchema,
  LoginCommandSchema,
  LoginWithCredentialsCommandSchema,
  LogoutCommandSchema,
} from "@client/v1/command/command_pb";
import { adminClient } from "./client";
import { queryClient } from "../main";
import type { Station } from "@client/v1/admin/station_pb";
import type { Team } from "@client/v1/admin/team_pb";

type ActionField = {
  key: string;
  label: string;
  type: "text" | "password";
  placeholder?: string;
  required: boolean;
};

export type StationTarget = Station & { team?: Team };

export type StationAction = {
  key: string;
  name: string;
  description: string;
  target: "single" | "multiple" | "both";
  fields: ActionField[];
  type?: "normal" | "danger";
  execute: (
    stations: StationTarget[],
    values: Record<string, string>,
    register?: (id: string, ips: string[], command: string) => void,
  ) => Promise<void> | void;
};

export const STATION_ACTIONS: StationAction[] = [
  {
    key: "loginWithCredentials",
    name: "Login with Credentials",
    target: "both",
    description:
      "Send a login command with username and password to the selected stations.",
    fields: [
      {
        key: "username",
        label: "Username",
        type: "text",
        placeholder: "Enter username",
        required: true,
      },
      {
        key: "password",
        label: "Password",
        type: "password",
        placeholder: "Enter password",
        required: true,
      },
    ],
    execute: (stations, values) =>
      adminClient.sendCommand(
        stations.map((s) => s.ip),
        {
          case: "loginWithCredentials",
          value: create(LoginWithCredentialsCommandSchema, {
            username: values.username,
            password: values.password,
          }),
        },
      ),
  },
  {
    key: "login",
    name: "Login",
    target: "both",
    description: "Send a login command to the selected stations.",
    fields: [],
    execute: (stations) =>
      adminClient.sendCommand(
        stations.map((s) => s.ip),
        {
          case: "login",
          value: create(LoginCommandSchema),
        },
      ),
  },
  {
    key: "logout",
    name: "Logout",
    target: "both",
    description: "Send a logout command to the selected stations.",
    fields: [],
    execute: (stations) =>
      adminClient.sendCommand(
        stations.map((s) => s.ip),
        {
          case: "logout",
          value: create(LogoutCommandSchema),
        },
      ),
  },
  {
    key: "custom",
    name: "Remote Command",
    target: "multiple",
    description:
      "Send a remote command to be executed to the selected stations.",
    fields: [
      {
        key: "command",
        label: "Command",
        type: "text",
        placeholder: "Enter command",
        required: true,
      },
    ],
    execute: (stations, values, register) => {
      if (!values) {
        return;
      }
      const ips = stations.map((s) => s.ip);
      const id = crypto.randomUUID();
      register?.(id, ips, values.command);
      return adminClient.sendCommand(ips, {
        case: "custom",
        value: create(CustomCommandSchema, {
          id,
          command: values.command,
        }),
      });
    },
  },
  {
    key: "exportYamlInventory",
    name: "Export YAML Inventory",
    target: "multiple",
    description:
      "Download a YAML-formatted Ansible inventory with hosts in the 'contest' group.",
    fields: [],
    execute: (stations) => {
      const yamlHeader = "contest:\n  hosts:\n";
      const hostLines = stations.map((s) => `    ${s.ip}:`).join("\n");
      const content = yamlHeader + hostLines;

      const blob = new Blob([content], { type: "text/yaml" });

      const url = window.URL.createObjectURL(blob);

      const link = document.createElement("a");
      link.href = url;
      link.setAttribute("download", "inventory.yml");

      document.body.appendChild(link);
      link.click();

      link.parentNode?.removeChild(link);
      window.URL.revokeObjectURL(url);
    },
  },
  {
    key: "copyCssh",
    name: "Copy ClusterSSH Command",
    target: "multiple",
    description:
      "Copy a 'cssh' command for all selected stations to your clipboard.",
    fields: [
      {
        key: "user",
        label: "User",
        type: "text",
        placeholder: "Enter a user",
        required: false,
      },
    ],
    execute: (stations, values) => {
      const ips = stations.map((s) => s.ip);
      let command;
      if (values.user?.length > 0) {
        command = `cssh -l ${values.user} ${ips.join(" ")}`;
      } else {
        command = `cssh ${ips.join(" ")}`;
      }
      navigator.clipboard.writeText(command);
    },
  },
  {
    key: "deleteStation",
    name: "Delete station",
    target: "single",
    description: "Permanently deletes the selected stations from the system",
    fields: [],
    type: "danger",
    execute: async (stations) => {
      if (stations.length != 1) {
        console.error('deleteStation is a "single" action');
      }
      const station = stations[0];
      await adminClient.deleteStation(station.ip);
      queryClient.invalidateQueries({ queryKey: ["stations"] });
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  },
  {
    key: "assignTeam",
    name: "Assign team",
    target: "both",
    description: "Tries to assign an available team to every selected station",
    fields: [],
    execute: async (stations) => {
      const unassigned = stations.filter((s) => !s.team);
      if (unassigned.length === 0) return;
      await adminClient.assignTeam(unassigned.map((s) => s.ip));
      queryClient.invalidateQueries({ queryKey: ["stations"] });
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  },
];
