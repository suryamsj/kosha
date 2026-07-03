import { invoke } from "@tauri-apps/api/core";

export interface KeyInfo {
  name: string;
  keyType: string;
  fingerprint: string;
  createdAt: number;
  hasPrivate: boolean;
}

interface ListKeysResponse {
  sshDirMissing: boolean;
  keys: KeyInfo[];
}

class KeysStore {
  keys = $state<KeyInfo[]>([]);
  sshDirMissing = $state(false);
  error = $state<string | null>(null);
  loading = $state(false);

  async refresh() {
    this.loading = true;
    this.error = null;
    try {
      const res = await invoke<ListKeysResponse>("list_keys");
      this.keys = res.keys;
      this.sshDirMissing = res.sshDirMissing;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async createSshDir() {
    await invoke("ensure_ssh_dir");
    await this.refresh();
  }

  async generate(name: string, passphrase: string) {
    await invoke<KeyInfo>("generate_key", {
      name,
      passphrase: passphrase || null,
    });
    await this.refresh();
  }

  async remove(name: string) {
    await invoke("delete_key", { name });
    await this.refresh();
  }
}

export const keysStore = new KeysStore();
