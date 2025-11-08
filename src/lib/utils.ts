import { invoke } from '@tauri-apps/api/core';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export async function sudo() {
  const res = await invoke<boolean>('sudo');

  if (!res) {
    // toast.error('授权失败');
    throw new Error('授权失败');
  }

  return true;
}
