import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { WithoutChild, WithoutChildren, WithoutChildrenOrChild } from "bits-ui";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export type WithElementRef<T, El extends HTMLElement = HTMLElement> = T & {
  ref?: El | null;
};

export type { WithoutChild, WithoutChildren, WithoutChildrenOrChild };
