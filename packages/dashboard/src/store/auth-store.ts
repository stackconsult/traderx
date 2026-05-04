import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User } from "@/types/store";

interface AuthState {
  // State
  token: string | null;
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  setToken: (token: string) => void;
  setUser: (user: User) => void;
  login: (token: string, user: User) => void;
  logout: () => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      // Initial state
      token: null,
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      // Actions
      setToken: (token) =>
        set({
          token,
          isAuthenticated: !!token,
        }),

      setUser: (user) => set({ user }),

      login: (token, user) =>
        set({
          token,
          user,
          isAuthenticated: true,
          error: null,
        }),

      logout: () =>
        set({
          token: null,
          user: null,
          isAuthenticated: false,
          error: null,
        }),

      setLoading: (isLoading) => set({ isLoading }),

      setError: (error) => set({ error }),

      clearError: () => set({ error: null }),
    }),
    {
      name: "traderx-auth",
      partialize: (state) => ({ token: state.token, user: state.user }),
    }
  )
);
