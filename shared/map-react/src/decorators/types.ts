import type { ReactNode } from "react";
import type { Seat } from "../types";

export type SeatOverlayRenderer = (seat: Seat) => ReactNode;
