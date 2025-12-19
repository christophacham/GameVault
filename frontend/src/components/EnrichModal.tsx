'use client';

import { useState } from 'react';
import { enrichGames, exportGames, importGames, EnrichResult, ExportResult, ImportResult } from '@/lib/api';

interface EnrichModalProps {
  isOpen: boolean;
  onClose: () => void;
  onComplete: () => void;
}

type Operation = 'enrich' | 'export' | 'import' | null;

export function EnrichModal({ isOpen, onClose, onComplete }: EnrichModalProps) {
  const [operation, setOperation] = useState<Operation>(null);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [result, setResult] = useState<string | null>(null);

  const handleEnrichAll = async () => {
    setOperation('enrich');
    setProgress(null);
    setResult(null);

    try {
      let totalEnriched = 0;
      let totalFailed = 0;
      let remaining = 1;
      let total = 0;

      while (remaining > 0) {
        const res: EnrichResult = await enrichGames();
        totalEnriched += res.enriched;
        totalFailed += res.failed;
        remaining = res.remaining;
        total = res.total;

        setProgress({
          done: total - remaining,
          total: total
        });
      }

      setResult(`Enrichment complete: ${totalEnriched} enriched, ${totalFailed} failed`);
    } catch (err) {
      setResult('Enrichment failed. Check console for details.');
      console.error(err);
    } finally {
      setOperation(null);
      setProgress(null);
    }
  };

  const handleExport = async () => {
    setOperation('export');
    setResult(null);

    try {
      const res: ExportResult = await exportGames();
      setResult(`Export complete: ${res.exported} exported, ${res.skipped} skipped, ${res.failed} failed`);
    } catch (err) {
      setResult('Export failed. Check console for details.');
      console.error(err);
    } finally {
      setOperation(null);
    }
  };

  const handleImport = async () => {
    setOperation('import');
    setResult(null);

    try {
      const res: ImportResult = await importGames();
      setResult(`Import complete: ${res.imported} imported, ${res.skipped} skipped (newer in DB), ${res.not_found} no file, ${res.failed} failed`);
    } catch (err) {
      setResult('Import failed. Check console for details.');
      console.error(err);
    } finally {
      setOperation(null);
    }
  };

  const handleClose = () => {
    if (result) {
      onComplete();
    }
    setResult(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50">
      <div className="bg-gv-card rounded-xl p-6 w-full max-w-md shadow-2xl">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold text-white">Enrich Library</h2>
          <button
            onClick={handleClose}
            disabled={!!operation}
            className="text-gray-400 hover:text-white disabled:opacity-50"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Result Message */}
        {result && (
          <div className={`mb-4 p-3 rounded-lg ${result.includes('failed') ? 'bg-red-900/50 text-red-200' : 'bg-green-900/50 text-green-200'}`}>
            {result}
          </div>
        )}

        {/* Progress */}
        {operation && (
          <div className="mb-4 p-3 bg-gv-hover rounded-lg">
            <div className="flex items-center gap-3">
              <svg className="animate-spin w-5 h-5 text-gv-accent" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
              </svg>
              <span className="text-white">
                {operation === 'enrich' ? 'Enriching' : operation === 'export' ? 'Exporting' : 'Importing'}...
                {progress && ` ${progress.done}/${progress.total}`}
              </span>
            </div>
          </div>
        )}

        {/* Action Buttons */}
        <div className="space-y-3">
          <button
            onClick={handleEnrichAll}
            disabled={!!operation}
            className="w-full px-4 py-3 bg-green-600 hover:bg-green-500 disabled:bg-green-600/50 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            Enrich All Games
          </button>

          <button
            onClick={handleExport}
            disabled={!!operation}
            className="w-full px-4 py-3 bg-blue-600 hover:bg-blue-500 disabled:bg-blue-600/50 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v4a2 2 0 002 2h12a2 2 0 002-2v-4m-4-4l-4-4m0 0 l-4 4m4-4v12" />
            </svg>
            Export Metadata to Files
          </button>

          <button
            onClick={handleImport}
            disabled={!!operation}
            className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-500 disabled:bg-purple-600/50 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v4a2 2 0 002 2h12a2 2 0 002-2v-4m4-8l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Import Metadata from Files
          </button>
        </div>

        <p className="mt-4 text-sm text-gray-500">
          Enrich fetches metadata from Steam. Export saves metadata to .gamevault folders. Import reads metadata from .gamevault folders (only imports if file is newer than database).
        </p>
      </div>
    </div>
  );
}
