import { invoke } from "@tauri-apps/api/core";

class TagsStore {
  tags = $state<Record<string, string[]>>({});
  error = $state<string | null>(null);

  async refresh() {
    this.error = null;
    try {
      this.tags = await invoke<Record<string, string[]>>("list_tags");
    } catch (e) {
      this.error = String(e);
    }
  }

  async setTags(aliasKey: string, tags: string[]) {
    this.error = null;
    try {
      await invoke("set_tags", { aliasKey, tags });
      await this.refresh();
    } catch (e) {
      this.error = String(e);
    }
  }
}

export const tagsStore = new TagsStore();
