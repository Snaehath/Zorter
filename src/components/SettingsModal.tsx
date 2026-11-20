import React, { useEffect, useState } from 'react';
import { X, Check, Server, Sliders, Shield } from 'lucide-react';
import type { AppSettings, ModelInfo } from '../types';
import { api } from '../services/api';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSettingsSaved: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  onSettingsSaved,
}) => {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [isSaving, setIsSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadData();
    }
  }, [isOpen]);

  const loadData = async () => {
    try {
      const [currSettings, models] = await Promise.all([
        api.getSettings(),
        api.listAvailableModels().catch(() => []),
      ]);
      setSettings(currSettings);
      setAvailableModels(models);
    } catch (e) {
      console.error('Failed to load settings or models', e);
    }
  };

  if (!isOpen || !settings) return null;

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await api.saveSettings(settings);
      setStatusMessage('Settings saved successfully!');
      setTimeout(() => {
        setStatusMessage(null);
        onSettingsSaved();
        onClose();
      }, 800);
    } catch (e) {
      setStatusMessage(`Error saving settings: ${e}`);
    } finally {
      setIsSaving(false);
    }
  };

  const visionCandidates = availableModels.filter((m) => m.is_vision);
  const embeddingCandidates = availableModels.filter((m) => m.is_embedding);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div className="w-full max-w-xl bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        {/* Header */}
        <div className="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <Sliders className="w-5 h-5 text-indigo-400" />
            <h2 className="text-base font-semibold text-white">Application Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Form Body */}
        <div className="p-6 overflow-y-auto space-y-6 text-sm">
          {/* Hardware & GPU Notice */}
          <div className="p-3.5 rounded-xl bg-indigo-950/40 border border-indigo-800/40 flex items-start gap-3">
            <Shield className="w-5 h-5 text-indigo-400 shrink-0 mt-0.5" />
            <div className="text-xs text-indigo-200 leading-relaxed">
              <span className="font-semibold text-indigo-100">Local-First Privacy Architecture:</span> Files never leave this machine. All vision reasoning, embeddings, and BLAKE3 hashes are calculated locally.
            </div>
          </div>

          {/* AI Runtime Endpoint */}
          <div className="space-y-2">
            <label className="block text-xs font-semibold text-slate-300 uppercase tracking-wider">
              Ollama API Endpoint
            </label>
            <div className="flex items-center gap-2">
              <Server className="w-4 h-4 text-slate-400 shrink-0" />
              <input
                type="text"
                value={settings.ollama_endpoint}
                onChange={(e) => setSettings({ ...settings, ollama_endpoint: e.target.value })}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200 focus:outline-none focus:border-indigo-500 font-mono text-xs"
                placeholder="http://127.0.0.1:11434"
              />
            </div>
          </div>

          {/* Vision Model Selection */}
          <div className="space-y-2">
            <label className="block text-xs font-semibold text-slate-300 uppercase tracking-wider">
              Preferred Vision Model
            </label>
            <select
              value={settings.preferred_vision_model}
              onChange={(e) => setSettings({ ...settings, preferred_vision_model: e.target.value })}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              {visionCandidates.length > 0 ? (
                visionCandidates.map((m) => (
                  <option key={m.name} value={m.name}>
                    {m.name} ({(m.size_bytes / (1024 * 1024 * 1024)).toFixed(1)} GB)
                  </option>
                ))
              ) : (
                <>
                  <option value="qwen3.5:4b">qwen3.5:4b (Installed)</option>
                  <option value="ministral-3:3b">ministral-3:3b (Installed)</option>
                  <option value="gemma-writer:latest">gemma-writer:latest (Installed)</option>
                </>
              )}
            </select>
            <p className="text-[11px] text-slate-400">
              Understands subjects, aesthetic style, environments, lighting, and colors.
            </p>
          </div>

          {/* Embedding Model Selection */}
          <div className="space-y-2">
            <label className="block text-xs font-semibold text-slate-300 uppercase tracking-wider">
              Preferred Embedding Model
            </label>
            <select
              value={settings.preferred_embedding_model}
              onChange={(e) => setSettings({ ...settings, preferred_embedding_model: e.target.value })}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              {embeddingCandidates.length > 0 ? (
                embeddingCandidates.map((m) => (
                  <option key={m.name} value={m.name}>
                    {m.name} ({(m.size_bytes / (1024 * 1024)).toFixed(0)} MB)
                  </option>
                ))
              ) : (
                <>
                  <option value="embeddinggemma-2:270m">embeddinggemma-2:270m (Installed)</option>
                </>
              )}
            </select>
            <p className="text-[11px] text-slate-400">
              Converts descriptions and user search queries into vector embeddings for semantic search.
            </p>
          </div>

          {/* Naming Convention */}
          <div className="space-y-2">
            <label className="block text-xs font-semibold text-slate-300 uppercase tracking-wider">
              Default Filename Convention
            </label>
            <div className="grid grid-cols-3 gap-2">
              {[
                { id: 'clean_snake_case', label: 'clean_snake_case', eg: 'cyberpunk_city.png' },
                { id: 'kebab-case', label: 'kebab-case', eg: 'cyberpunk-city.png' },
                { id: 'title_case', label: 'Title_Case', eg: 'Cyberpunk_City.png' },
              ].map((conv) => (
                <button
                  key={conv.id}
                  type="button"
                  onClick={() => setSettings({ ...settings, default_naming_convention: conv.id })}
                  className={`p-2.5 rounded-lg border text-left transition ${
                    settings.default_naming_convention === conv.id
                      ? 'border-indigo-500 bg-indigo-500/15 text-white'
                      : 'border-slate-800 bg-slate-950 text-slate-400 hover:border-slate-700'
                  }`}
                >
                  <div className="text-xs font-medium">{conv.label}</div>
                  <div className="text-[10px] text-slate-400 mt-1 font-mono truncate">{conv.eg}</div>
                </button>
              ))}
            </div>
          </div>

          {/* Organization Hierarchy */}
          <div className="space-y-2">
            <label className="block text-xs font-semibold text-slate-300 uppercase tracking-wider">
              Default Organization Behavior
            </label>
            <select
              value={settings.default_organization_behavior}
              onChange={(e) => setSettings({ ...settings, default_organization_behavior: e.target.value })}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              <option value="by_category">By Category & Subcategory (e.g. Wallpapers/Cyberpunk/)</option>
              <option value="category_and_style">By Category & Visual Style (e.g. Wallpapers/Digital Art/)</option>
              <option value="flat_rename">Flat Rename in Current Directory (No Subfolders)</option>
            </select>
          </div>

          {/* Inference Concurrency */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="block text-xs font-semibold text-slate-300 uppercase tracking-wider">
                Inference Concurrency (VRAM Safety)
              </label>
              <span className="text-xs font-mono text-indigo-400 font-bold">
                {settings.inference_concurrency} worker
              </span>
            </div>
            <input
              type="range"
              min={1}
              max={3}
              step={1}
              value={settings.inference_concurrency}
              onChange={(e) => setSettings({ ...settings, inference_concurrency: parseInt(e.target.value) })}
              className="w-full accent-indigo-500 cursor-pointer"
            />
            <p className="text-[11px] text-slate-400">
              Kept at 1 by default to prevent VRAM memory pressure on RTX 3050 4GB Laptop GPUs.
            </p>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-slate-800 flex items-center justify-between bg-slate-950/50">
          <div className="text-xs text-emerald-400 font-medium">{statusMessage}</div>
          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-4 py-2 rounded-lg text-xs font-medium text-slate-400 hover:text-white transition"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={isSaving}
              className="px-5 py-2 rounded-lg text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white transition flex items-center gap-1.5 shadow-lg shadow-indigo-600/20"
            >
              <Check className="w-3.5 h-3.5" />
              Save Preferences
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
