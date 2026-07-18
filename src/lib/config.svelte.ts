import { invoke } from "@tauri-apps/api/core";
import { keysStore } from "$lib/keys.svelte";

export interface HostEntry {
  aliases: string[];
  hostName: string | null;
  user: string | null;
  port: string | null;
  identityFile: string | null;
}

export interface ImportPreview {
  accepted: HostEntry[];
  skipped: string[];
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

  async addHost(
    aliases: string[],
    hostName: string | null,
    user: string | null,
    port: string | null,
    identityFile: string | null,
  ) {
    await invoke("add_host", { aliases, hostName, user, port, identityFile });
    await this.refresh();
    await keysStore.refresh();
  }

  async editHost(
    originalAliases: string[],
    aliases: string[],
    hostName: string | null,
    user: string | null,
    port: string | null,
    identityFile: string | null,
  ) {
    await invoke("edit_host", {
      originalAliases,
      aliases,
      hostName,
      user,
      port,
      identityFile,
    });
    await this.refresh();
    await keysStore.refresh();
  }

  async removeHost(aliases: string[]) {
    await invoke("delete_host", { aliases });
    await this.refresh();
    await keysStore.refresh();
  }

  async exportHosts(): Promise<string> {
    return invoke<string>("export_hosts");
  }

  async previewImport(text: string): Promise<ImportPreview> {
    return invoke<ImportPreview>("preview_import", { text });
  }

  async importHosts(text: string): Promise<ImportPreview> {
    const result = await invoke<ImportPreview>("import_hosts", { text });
    await this.refresh();
    await keysStore.refresh();
    return result;
  }
}

export const configStore = new ConfigStore();
