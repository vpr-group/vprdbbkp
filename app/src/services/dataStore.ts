import { createId } from "@paralleldrive/cuid2";
import { load, Store } from "@tauri-apps/plugin-store";

export interface S3StorageConfig {
  type: "S3";
  region: string;
  bucket: string;
  endpoint: string;
  accessKey: string;
  secretKey: string;
}

export type StorageConfig = S3StorageConfig;

export interface Project {
  id: string;
  name: string;
  config: StorageConfig;
}

export interface PostgresConfig {
  type: "postgres";
  database: string;
  host: string;
  port: number;
  username: string;
  password: string;
}

export type WorkspaceConfig = PostgresConfig;

export interface Workspace {
  id: string;
  name: string;
  projectId: string;
  config: WorkspaceConfig;
}

export class StoreService {
  private internalStore: Store | undefined;
  private initializationPromise: Promise<void>;

  constructor() {
    this.initializationPromise = this.initialize();
  }

  private async initialize() {
    this.internalStore = await load("store.json", { autoSave: false });
  }

  async waitForInitialized(): Promise<void> {
    return this.initializationPromise;
  }

  get store() {
    if (!this.internalStore) throw new Error("Store is not initialized");
    return this.internalStore;
  }

  // Project operations
  async getProjects(): Promise<Project[]> {
    const projects =
      ((await this.store.get("projects")) as Record<string, Project>) || {};
    return Object.values(projects);
  }

  async getProject(id: string): Promise<Project | null> {
    const projects =
      ((await this.store.get("projects")) as Record<string, Project>) || {};
    return projects[id] || null;
  }

  async saveProject(
    project: Omit<Project, "id"> & { id?: string }
  ): Promise<Project> {
    const projects =
      ((await this.store.get("projects")) as Record<string, Project>) || {};

    const completeProject: Project = {
      ...project,
      id: project.id || createId(), // Generate ID if not provided
    };

    const updatedProjects = {
      ...projects,
      [completeProject.id]: completeProject,
    };

    await this.store.set("projects", updatedProjects);
    await this.store.save();

    return completeProject;
  }

  async deleteProject(id: string): Promise<void> {
    const projects =
      ((await this.store.get("projects")) as Record<string, Project>) || {};
    const workspaces =
      ((await this.store.get("workspaces")) as Record<string, Workspace>) || {};

    // Delete the project
    if (projects[id]) {
      delete projects[id];
      await this.store.set("projects", projects);
    }

    // Also delete related workspaces
    const updatedWorkspaces = Object.values(workspaces)
      .filter((workspace) => workspace.projectId !== id)
      .reduce((acc, workspace) => {
        acc[workspace.id] = workspace;
        return acc;
      }, {} as Record<string, Workspace>);

    await this.store.set("workspaces", updatedWorkspaces);
    await this.store.save();
  }

  // Workspace operations
  async getWorkspaces(projectId?: string): Promise<Workspace[]> {
    const workspaces =
      ((await this.store.get("workspaces")) as Record<string, Workspace>) || {};

    if (projectId) {
      return Object.values(workspaces).filter(
        (workspace) => workspace.projectId === projectId
      );
    }

    return Object.values(workspaces);
  }

  async getWorkspace(id: string): Promise<Workspace | null> {
    const workspaces =
      ((await this.store.get("workspaces")) as Record<string, Workspace>) || {};
    return workspaces[id] || null;
  }

  async saveWorkspace(
    workspace: Omit<Workspace, "id"> & { id?: string }
  ): Promise<Workspace> {
    // Verify the project exists
    const projects =
      ((await this.store.get("projects")) as Record<string, Project>) || {};
    if (!projects[workspace.projectId]) {
      throw new Error(`Project with ID ${workspace.projectId} not found`);
    }

    const workspaces =
      ((await this.store.get("workspaces")) as Record<string, Workspace>) || {};

    const completeWorkspace: Workspace = {
      ...workspace,
      id: workspace.id || createId(), // Generate ID if not provided
    };

    const updatedWorkspaces = {
      ...workspaces,
      [completeWorkspace.id]: completeWorkspace,
    };

    await this.store.set("workspaces", updatedWorkspaces);
    await this.store.save();

    return completeWorkspace;
  }

  async deleteWorkspace(id: string): Promise<void> {
    const workspaces =
      ((await this.store.get("workspaces")) as Record<string, Workspace>) || {};

    if (workspaces[id]) {
      delete workspaces[id];
      await this.store.set("workspaces", workspaces);
      await this.store.save();
    }
  }
}
