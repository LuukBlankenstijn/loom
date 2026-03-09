import { create } from "@bufbuild/protobuf";
import {
  CustomCommandSchema,
  LoginCommandSchema,
  LoginWithCredentialsCommandSchema,
  LogoutCommandSchema,
} from "@client/v1/command/command_pb";
import { adminClient } from "./client";
import type { Station } from "@client/v1/admin/admin_pb";
import { queryClient } from "../main";

type ActionField = {
  key: string;
  label: string;
  type: "text" | "password";
  placeholder?: string;
  required: boolean;
};

export type StationAction = {
  key: string;
  name: string;
  description: string;
  allowSingle: boolean;
  fields: ActionField[];
  type?: "normal" | "danger";
  execute: (
    stations: Station[],
    values: Record<string, string>,
    register?: (id: string, ips: string[], command: string) => void,
  ) => Promise<void> | void;
};

export const STATION_ACTIONS: StationAction[] = [
  {
    key: "loginWithCredentials",
    name: "Login with Credentials",
    allowSingle: true,
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
    allowSingle: true,
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
    allowSingle: true,
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
    allowSingle: false,
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
    allowSingle: false,
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
    allowSingle: false,
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
    allowSingle: true,
    description: "Permanently deletes the selected stations from the system",
    fields: [],
    type: "danger",
    execute: async (stations) => {
      await adminClient.deleteStations(stations.map((s) => s.id));
      queryClient.invalidateQueries({ queryKey: ["stations"] });
    },
  },
];
