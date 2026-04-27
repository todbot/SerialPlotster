import { useCallback, useRef, useState } from 'react';
import { RingStore } from '../store/RingStore';

export function useRingBuffer(capacity = 100_000) {
  const store = useRef(new RingStore(capacity)).current;
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
