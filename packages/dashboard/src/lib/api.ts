import axios, { AxiosInstance, AxiosError, AxiosRequestConfig } from "axios";
import { useAuthStore } from "@/store/auth-store";

// API Types
export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

export interface ApiResponse<T> {
  data: T;
  success: boolean;
  error?: ApiError;
}

// Create axios instance
const apiClient: AxiosInstance = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080",
  timeout: 10000,
  headers: {
    "Content-Type": "application/json",
  },
});

// Request interceptor - attach JWT token
apiClient.interceptors.request.use(
  (config) => {
    const token = useAuthStore.getState().token;
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// Response interceptor - handle errors
apiClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError<ApiError>) => {
    if (error.response) {
      const { status, data } = error.response;

      // Handle 401 Unauthorized
      if (status === 401) {
        useAuthStore.getState().logout();
        window.location.href = "/login";
      }

      // Handle 429 Rate Limit
      if (status === 429) {
        console.warn("Rate limited, please retry later");
      }

      return Promise.reject({
        code: data?.code || "UNKNOWN_ERROR",
        message: data?.message || "An error occurred",
        details: data?.details,
      });
    }

    if (error.request) {
      // Network error
      return Promise.reject({
        code: "NETWORK_ERROR",
        message: "Network error, please check your connection",
      });
    }

    return Promise.reject({
      code: "REQUEST_ERROR",
      message: error.message,
    });
  }
);

// API wrapper with retry logic
async function apiRequest<T>(
  method: "get" | "post" | "put" | "delete",
  url: string,
  data?: unknown,
  config?: AxiosRequestConfig,
  retries = 3
): Promise<T> {
  try {
    const response = await apiClient[method](url, data, config);
    return response.data;
  } catch (error) {
    if (retries > 0 && isRetryableError(error)) {
      await delay(1000 * (4 - retries)); // Exponential backoff
      return apiRequest(method, url, data, config, retries - 1);
    }
    throw error;
  }
}

function isRetryableError(error: unknown): boolean {
  if (axios.isAxiosError(error)) {
    return (
      !error.response || // Network error
      error.response.status >= 500 // Server error
    );
  }
  return false;
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// API methods
export const api = {
  get: <T>(url: string, config?: AxiosRequestConfig) =>
    apiRequest<T>("get", url, undefined, config),

  post: <T>(url: string, data?: unknown, config?: AxiosRequestConfig) =>
    apiRequest<T>("post", url, data, config),

  put: <T>(url: string, data?: unknown, config?: AxiosRequestConfig) =>
    apiRequest<T>("put", url, data, config),

  delete: <T>(url: string, config?: AxiosRequestConfig) =>
    apiRequest<T>("delete", url, undefined, config),
};

export default apiClient;
