import { invoke } from "@tauri-apps/api/core";

export interface HostEntry {
  aliases: string[];
  hostName: string | null;
  user: string | null;
  port: string | null;
  identityFile: string | null;
}

class ConfigStore {
  hosts = $state<HostEntry[]>([]);
  error = $state<string | null>(null);
  loading = $state(false);

  async refresh() {
    this.loading = true;
    this.error = null;
    try {
      this.hosts = await invoke<HostEntry[]>("list_hosts");
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }
}

export const configStore = new ConfigStore();
