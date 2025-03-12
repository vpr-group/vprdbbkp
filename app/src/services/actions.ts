import type { StorageProvider } from "./store";

export class ActionsService {
  async listBackups(storageProvider: StorageProvider): Promise<any> {
    console.log(storageProvider);
  }
}
