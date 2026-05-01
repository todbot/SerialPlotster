import { useCallback, useRef, useState } from 'react';
import { RingStore } from '../store/RingStore';

export function useRingBuffer() {
  const store = useRef(new RingStore()).current;
  const [seriesKeys, setSeriesKeys] = useState<string[]>([]);

  const onNewSeries = useCallback(() => {
    setSeriesKeys([...store.seriesKeys]);
  }, [store]);

  const clearStore = useCallback(() => {
    store.clear();
    setSeriesKeys([]);
  }, [store]);

  return { store, seriesKeys, onNewSeries, clearStore };
}
