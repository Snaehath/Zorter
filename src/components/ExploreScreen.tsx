import React, { useState, useEffect } from 'react';
import {
  Search,
  SlidersHorizontal,
  Compass,
  Copy,
  Sparkles,
  Maximize2,
  X,
  FileText,
} from 'lucide-react';
import type {
  DuplicateGroup,
  FileAnalysis,
  IndexedFile,
  SearchFilters,
  SearchResultItem,
} from '../types';
import { api } from '../services/api';

interface ExploreScreenProps {
  files: IndexedFile[];
  analyses: FileAnalysis[];
}

export const ExploreScreen: React.FC<ExploreScreenProps> = ({ files, analyses }) => {
  const [activeTab, setActiveTab] = useState<'semantic' | 'duplicates' | 'browse'>('semantic');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedStyle, setSelectedStyle] = useState<string>('all');
  const [selectedOrientation, setSelectedOrientation] = useState<string>('all');
  const [searchResults, setSearchResults] = useState<SearchResultItem[]>([]);
  const [duplicateGroups, setDuplicateGroups] = useState<DuplicateGroup[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [selectedFileForDetail, setSelectedFileForDetail] = useState<{
    file: IndexedFile;
    analysis?: FileAnalysis | null;
  } | null>(null);

  // Analysis lookup map
  const analysisMap = new Map<string, FileAnalysis>();
  analyses.forEach((a) => analysisMap.set(a.file_id, a));

  // Extract unique categories and styles for filter dropdowns
  const categories = Array.from(new Set(analyses.map((a) => a.category))).filter(Boolean);
  const styles = Array.from(new Set(analyses.map((a) => a.visual_style))).filter(Boolean);

  useEffect(() => {
    // Initial browse load
    handleSearch('');
    loadDuplicates();
  }, [files, analyses]);

  const loadDuplicates = async () => {
    try {
      const dups = await api.getDuplicateFiles();
      setDuplicateGroups(dups);
    } catch (e) {
      console.error('Failed to load duplicate files', e);
    }
  };

  const handleSearch = async (queryText: string) => {
    setIsSearching(true);
    try {
      const filters: SearchFilters = {
        category: selectedCategory !== 'all' ? selectedCategory : undefined,
        visual_style: selectedStyle !== 'all' ? selectedStyle : undefined,
        orientation: selectedOrientation !== 'all' ? selectedOrientation : undefined,
      };

      const results = await api.searchSemantic(queryText, filters, 60);
      setSearchResults(results);
    } catch (e) {
      console.error('Semantic search failed', e);
    } finally {
      setIsSearching(false);
    }
  };

  const handleFindSimilar = async (fileId: string) => {
    setIsSearching(true);
    try {
      const similar = await api.findSimilarImages(fileId, 24);
      setSearchResults(similar);
      setActiveTab('semantic');
      setSearchQuery(`Similar to ${files.find((f) => f.id === fileId)?.current_name || 'selected image'}`);
    } catch (e) {
      console.error('Find similar failed', e);
    } finally {
      setIsSearching(false);
    }
  };

  const sampleQueries = [
    'Futuristic cyberpunk city at night with neon lighting',
    'Serene mountain landscape during golden hour',
    'Dark fantasy digital illustration with glowing magic',
    'Minimalist space wallpaper with planets',
    'Vibrant anime style character portrait',
  ];

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-16">
      {/* Top Search & Filter Bar */}
      <div className="p-6 rounded-2xl bg-slate-900/80 border border-slate-800 shadow-xl space-y-4">
        {/* Navigation Sub-Tabs */}
        <div className="flex items-center justify-between border-b border-slate-800 pb-4">
          <div className="flex items-center gap-2">
            <button
              onClick={() => setActiveTab('semantic')}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition ${
                activeTab === 'semantic'
                  ? 'bg-indigo-600 text-white shadow'
                  : 'text-slate-400 hover:text-white hover:bg-slate-800'
              }`}
            >
              <Sparkles className="w-4 h-4" />
              Semantic Search
            </button>

            <button
              onClick={() => setActiveTab('duplicates')}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition ${
                activeTab === 'duplicates'
                  ? 'bg-amber-600 text-white shadow'
                  : 'text-slate-400 hover:text-white hover:bg-slate-800'
              }`}
            >
              <Copy className="w-4 h-4" />
              Duplicate Detector ({duplicateGroups.length})
            </button>

            <button
              onClick={() => {
                setActiveTab('browse');
                setSearchQuery('');
                handleSearch('');
              }}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition ${
                activeTab === 'browse'
                  ? 'bg-cyan-600 text-white shadow'
                  : 'text-slate-400 hover:text-white hover:bg-slate-800'
              }`}
            >
              <Compass className="w-4 h-4" />
              All Indexed ({files.length})
            </button>
          </div>

          <span className="text-xs text-slate-400 hidden sm:inline">
            Local Vector Space: <span className="font-mono text-indigo-300">Gemma Embeddings</span>
          </span>
        </div>

        {/* Semantic Search Input */}
        {activeTab !== 'duplicates' && (
          <div className="space-y-3">
            <div className="relative">
              <Search className="w-5 h-5 text-slate-400 absolute left-4 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleSearch(searchQuery);
                }}
                placeholder="Ask what you're looking for (e.g. 'Show dark cyberpunk images with blue and purple neon')..."
                className="w-full bg-slate-950 border border-slate-800 rounded-xl pl-12 pr-28 py-3 text-sm text-slate-100 placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition shadow-inner"
              />
              <button
                onClick={() => handleSearch(searchQuery)}
                disabled={isSearching}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 px-4 py-1.5 rounded-lg text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white transition cursor-pointer"
              >
                {isSearching ? 'Searching...' : 'Search'}
              </button>
            </div>

            {/* Prompt Suggestions */}
            <div className="flex flex-wrap items-center gap-1.5 text-xs text-slate-400">
              <span className="text-slate-500 text-[11px]">Try searching:</span>
              {sampleQueries.map((q, idx) => (
                <button
                  key={idx}
                  onClick={() => {
                    setSearchQuery(q);
                    handleSearch(q);
                  }}
                  className="px-2.5 py-1 rounded-full bg-slate-950 hover:bg-slate-800 border border-slate-800 text-[11px] text-slate-300 hover:text-white transition cursor-pointer"
                >
                  {q}
                </button>
              ))}
            </div>

            {/* Filters Bar */}
            <div className="flex flex-wrap items-center gap-3 pt-2 text-xs">
              <div className="flex items-center gap-1.5 text-slate-400">
                <SlidersHorizontal className="w-3.5 h-3.5" />
                <span>Filters:</span>
              </div>

              {/* Category Filter */}
              <select
                value={selectedCategory}
                onChange={(e) => {
                  setSelectedCategory(e.target.value);
                  setTimeout(() => handleSearch(searchQuery), 0);
                }}
                className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1 text-slate-200 focus:outline-none focus:border-indigo-500"
              >
                <option value="all">All Categories</option>
                {categories.map((c) => (
                  <option key={c} value={c}>
                    {c}
                  </option>
                ))}
              </select>

              {/* Visual Style Filter */}
              <select
                value={selectedStyle}
                onChange={(e) => {
                  setSelectedStyle(e.target.value);
                  setTimeout(() => handleSearch(searchQuery), 0);
                }}
                className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1 text-slate-200 focus:outline-none focus:border-indigo-500"
              >
                <option value="all">All Visual Styles</option>
                {styles.map((s) => (
                  <option key={s} value={s}>
                    {s}
                  </option>
                ))}
              </select>

              {/* Orientation Filter */}
              <select
                value={selectedOrientation}
                onChange={(e) => {
                  setSelectedOrientation(e.target.value);
                  setTimeout(() => handleSearch(searchQuery), 0);
                }}
                className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1 text-slate-200 focus:outline-none focus:border-indigo-500"
              >
                <option value="all">All Orientations</option>
                <option value="landscape">Landscape</option>
                <option value="portrait">Portrait</option>
                <option value="square">Square</option>
              </select>
            </div>
          </div>
        )}
      </div>

      {/* Main Gallery / Duplicate Clusters */}
      {activeTab === 'duplicates' ? (
        /* Duplicate Clusters View */
        <div className="space-y-4">
          <div className="flex items-center justify-between text-xs text-slate-400">
            <span>
              Identified <span className="font-semibold text-white">{duplicateGroups.length}</span> duplicate content groups using BLAKE3 fingerprinting.
            </span>
          </div>

          {duplicateGroups.length === 0 ? (
            <div className="p-12 text-center text-xs text-slate-400 border border-dashed border-slate-800 rounded-2xl bg-slate-900/30">
              No identical duplicate files detected in currently scanned folders!
            </div>
          ) : (
            <div className="space-y-4">
              {duplicateGroups.map((group, gIdx) => (
                <div
                  key={group.hash}
                  className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-3"
                >
                  <div className="flex items-center justify-between text-xs">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-amber-300">
                        Group #{gIdx + 1}
                      </span>
                      <span className="text-slate-500">•</span>
                      <span className="font-mono text-slate-400">
                        BLAKE3: {group.hash.slice(0, 16)}...
                      </span>
                    </div>
                    <span className="text-amber-400 font-medium">
                      {(group.wasted_bytes / (1024 * 1024)).toFixed(2)} MB redundant
                    </span>
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
                    {group.files.map((file) => (
                      <div
                        key={file.id}
                        className="p-3 rounded-xl bg-slate-950 border border-slate-800/80 flex items-center gap-3 text-xs"
                      >
                        <div className="w-12 h-12 rounded-lg bg-slate-900 overflow-hidden shrink-0">
                          {file.thumbnail_url ? (
                            <img
                              src={file.thumbnail_url}
                              alt={file.current_name}
                              className="w-full h-full object-cover"
                            />
                          ) : (
                            <div className="w-full h-full flex items-center justify-center text-slate-600">
                              IMG
                            </div>
                          )}
                        </div>
                        <div className="min-w-0 flex-1">
                          <p className="font-medium text-slate-200 truncate">
                            {file.current_name}
                          </p>
                          <p className="text-[10px] text-slate-400 font-mono truncate">
                            {file.current_path}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      ) : (
        /* Image Results Grid */
        <div className="space-y-4">
          <div className="flex items-center justify-between text-xs text-slate-400">
            <span>
              Showing <span className="font-semibold text-white">{searchResults.length}</span> images
            </span>
          </div>

          {searchResults.length === 0 ? (
            <div className="p-12 text-center text-xs text-slate-400 border border-dashed border-slate-800 rounded-2xl bg-slate-900/30">
              No matching images found. Try a different query or adjust your filters.
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {searchResults.map((item) => {
                const analysis = item.analysis || analysisMap.get(item.file.id);

                return (
                  <div
                    key={item.file.id}
                    className="group rounded-2xl bg-slate-900 border border-slate-800 overflow-hidden flex flex-col hover:border-slate-700 hover:shadow-xl transition-all"
                  >
                    {/* Image Thumbnail Container */}
                    <div className="relative aspect-square bg-slate-950 overflow-hidden">
                      {item.file.thumbnail_url ? (
                        <img
                          src={item.file.thumbnail_url}
                          alt={item.file.current_name}
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                        />
                      ) : (
                        <div className="w-full h-full flex items-center justify-center text-slate-700">
                          No Thumbnail
                        </div>
                      )}

                      {/* Similarity Score Pill */}
                      {item.score < 1.0 && item.score > 0 && (
                        <div className="absolute top-2.5 right-2.5 px-2 py-0.5 rounded-full text-[10px] font-bold bg-indigo-600/90 text-white backdrop-blur shadow">
                          {(item.score * 100).toFixed(0)}% Match
                        </div>
                      )}

                      {/* Quick Hover Actions Overlay */}
                      <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2 p-4">
                        <button
                          onClick={() => handleFindSimilar(item.file.id)}
                          className="px-3 py-1.5 rounded-lg text-xs font-medium bg-indigo-600 hover:bg-indigo-500 text-white transition flex items-center gap-1.5 shadow cursor-pointer"
                        >
                          <Sparkles className="w-3.5 h-3.5" />
                          Similar
                        </button>
                        <button
                          onClick={() => setSelectedFileForDetail({ file: item.file, analysis })}
                          className="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 transition cursor-pointer"
                          title="View Details"
                        >
                          <Maximize2 className="w-4 h-4" />
                        </button>
                      </div>
                    </div>

                    {/* Meta Card Body */}
                    <div className="p-3.5 space-y-2 flex-1 flex flex-col justify-between text-xs">
                      <div>
                        <p className="font-semibold text-slate-200 truncate" title={item.file.current_name}>
                          {item.file.current_name}
                        </p>
                        {analysis && (
                          <p className="text-[11px] text-slate-400 line-clamp-2 mt-1 leading-relaxed">
                            {analysis.description}
                          </p>
                        )}
                      </div>

                      {/* Attributes Chips */}
                      {analysis && (
                        <div className="flex flex-wrap items-center gap-1 pt-2 border-t border-slate-800/60">
                          <span className="text-[9px] font-bold px-1.5 py-0.5 rounded bg-indigo-500/15 text-indigo-300">
                            {analysis.category}
                          </span>
                          {analysis.visual_style && (
                            <span className="text-[9px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400">
                              {analysis.visual_style}
                            </span>
                          )}
                          {analysis.lighting && (
                            <span className="text-[9px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400">
                              {analysis.lighting}
                            </span>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      {/* Image Detail Modal Drawer */}
      {selectedFileForDetail && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
          <div className="w-full max-w-3xl bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-[92vh]">
            {/* Header */}
            <div className="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <FileText className="w-4 h-4 text-indigo-400" />
                <h3 className="text-sm font-semibold text-white truncate max-w-md">
                  {selectedFileForDetail.file.current_name}
                </h3>
              </div>
              <button
                onClick={() => setSelectedFileForDetail(null)}
                className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* Modal Body */}
            <div className="p-6 overflow-y-auto space-y-6">
              {/* Preview Image */}
              <div className="w-full h-80 rounded-2xl bg-slate-950 border border-slate-800 overflow-hidden flex items-center justify-center">
                {selectedFileForDetail.file.thumbnail_url ? (
                  <img
                    src={selectedFileForDetail.file.thumbnail_url}
                    alt={selectedFileForDetail.file.current_name}
                    className="w-full h-full object-contain"
                  />
                ) : (
                  <div className="text-slate-600 text-xs">No preview available</div>
                )}
              </div>

              {/* AI Visual Intelligence Section */}
              {selectedFileForDetail.analysis ? (
                <div className="space-y-4">
                  <div className="space-y-1">
                    <span className="text-[11px] font-semibold uppercase text-indigo-400 tracking-wider">
                      AI Generated Description
                    </span>
                    <p className="text-sm text-slate-200 leading-relaxed">
                      {selectedFileForDetail.analysis.description}
                    </p>
                  </div>

                  <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
                    <div className="p-3 rounded-xl bg-slate-950 border border-slate-800">
                      <span className="text-slate-400 block text-[10px]">Category</span>
                      <span className="font-semibold text-slate-200">
                        {selectedFileForDetail.analysis.category}
                      </span>
                    </div>

                    <div className="p-3 rounded-xl bg-slate-950 border border-slate-800">
                      <span className="text-slate-400 block text-[10px]">Subcategory</span>
                      <span className="font-semibold text-slate-200">
                        {selectedFileForDetail.analysis.subcategory}
                      </span>
                    </div>

                    <div className="p-3 rounded-xl bg-slate-950 border border-slate-800">
                      <span className="text-slate-400 block text-[10px]">Visual Style</span>
                      <span className="font-semibold text-slate-200">
                        {selectedFileForDetail.analysis.visual_style}
                      </span>
                    </div>

                    <div className="p-3 rounded-xl bg-slate-950 border border-slate-800">
                      <span className="text-slate-400 block text-[10px]">Lighting</span>
                      <span className="font-semibold text-slate-200">
                        {selectedFileForDetail.analysis.lighting || 'Natural'}
                      </span>
                    </div>
                  </div>

                  {/* Colors & Subject Tags */}
                  <div className="space-y-2">
                    <span className="text-[11px] font-semibold uppercase text-slate-400 tracking-wider">
                      Color Palette & Subjects
                    </span>
                    <div className="flex flex-wrap items-center gap-1.5">
                      {selectedFileForDetail.analysis.colors.map((c, i) => (
                        <span
                          key={i}
                          className="px-2.5 py-1 rounded-full text-xs bg-slate-950 border border-slate-800 text-slate-300 flex items-center gap-1.5"
                        >
                          <span
                            className="w-2.5 h-2.5 rounded-full border border-slate-700"
                            style={{ backgroundColor: c.toLowerCase() }}
                          />
                          {c}
                        </span>
                      ))}
                      {selectedFileForDetail.analysis.subject.map((s, i) => (
                        <span
                          key={i}
                          className="px-2.5 py-1 rounded-full text-xs bg-indigo-500/10 border border-indigo-500/20 text-indigo-300"
                        >
                          {s}
                        </span>
                      ))}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-400">
                  This image has not been analyzed by local AI yet. Switch to "Analyze & Organize" to run local vision inference.
                </div>
              )}

              {/* Technical Metadata */}
              <div className="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-2 text-xs">
                <span className="font-semibold text-slate-300">File Technical Details</span>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-[11px] font-mono text-slate-400">
                  <div>Size: {(selectedFileForDetail.file.size / (1024 * 1024)).toFixed(2)} MB</div>
                  <div>Format: {selectedFileForDetail.file.type}</div>
                  <div className="truncate">Path: {selectedFileForDetail.file.current_path}</div>
                  <div className="truncate">BLAKE3: {selectedFileForDetail.file.hash}</div>
                </div>
              </div>
            </div>

            {/* Footer */}
            <div className="px-6 py-4 border-t border-slate-800 flex items-center justify-between bg-slate-950/60">
              <button
                onClick={() => {
                  const id = selectedFileForDetail.file.id;
                  setSelectedFileForDetail(null);
                  handleFindSimilar(id);
                }}
                className="px-4 py-2 rounded-xl text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white transition flex items-center gap-1.5"
              >
                <Sparkles className="w-3.5 h-3.5" />
                Find Similar Images
              </button>

              <button
                onClick={() => setSelectedFileForDetail(null)}
                className="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white transition"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
