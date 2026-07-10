export interface AppStatus {
  phase: "pre-alpha";
  scaffoldReady: boolean;
  captureEnabled: boolean;
  aiEnabled: boolean;
}

export interface DocReference {
  label: string;
  path: string;
  description: string;
}

export interface Principle {
  title: string;
  body: string;
}

export const appStatus: AppStatus = {
  phase: "pre-alpha",
  scaffoldReady: true,
  captureEnabled: true,
  aiEnabled: true
};

export const docReferences: DocReference[] = [
  {
    label: "Product Spec",
    path: "docs/PRODUCT_SPEC.md",
    description: "The user goal, MVP scope, trust rules, and demo acceptance."
  },
  {
    label: "Architecture",
    path: "docs/ARCHITECTURE.md",
    description: "The planned React, Tauri, Rust, service, and adapter boundaries."
  },
  {
    label: "Implementation Plan",
    path: "docs/IMPLEMENTATION_PLAN.md",
    description: "The phase-by-phase build order every implementation agent follows."
  },
  {
    label: "Security Policy",
    path: "docs/GIT_AND_SECURITY_POLICY.md",
    description: "What can be committed, what must never be committed, and checks."
  }
];

export const principles: Principle[] = [
  {
    title: "Selected regions only",
    body: "ScreenPebble must never imply whole-screen monitoring."
  },
  {
    title: "Low FPS by design",
    body: "The product starts at 1 FPS and caps the first public release at 5 FPS."
  },
  {
    title: "No captured history",
    body: "Frames, screenshots, previews, and OCR history are not persisted."
  },
  {
    title: "AI stays explicit",
    body: "A cropped image is sent only when the user asks about a selected region."
  }
];
