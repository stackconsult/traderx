import { useAuthStore } from "@/store/auth-store";

describe("auth-store", () => {
  beforeEach(() => {
    useAuthStore.setState({
      token: null,
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    });
  });

  it("logs in with token and user", () => {
    const user = { id: "u1", username: "trader", email: "t@x.com" };
    useAuthStore.getState().login("jwt-token-123", user);
    expect(useAuthStore.getState().token).toBe("jwt-token-123");
    expect(useAuthStore.getState().user?.username).toBe("trader");
    expect(useAuthStore.getState().isAuthenticated).toBe(true);
  });

  it("logs out and clears state", () => {
    const user = { id: "u1", username: "trader", email: "t@x.com" };
    useAuthStore.getState().login("jwt-token-123", user);
    useAuthStore.getState().logout();
    expect(useAuthStore.getState().token).toBeNull();
    expect(useAuthStore.getState().isAuthenticated).toBe(false);
    expect(useAuthStore.getState().user).toBeNull();
  });

  it("sets token and updates isAuthenticated", () => {
    useAuthStore.getState().setToken("new-token");
    expect(useAuthStore.getState().isAuthenticated).toBe(true);
  });

  it("sets error and clears it", () => {
    useAuthStore.getState().setError("Invalid credentials");
    expect(useAuthStore.getState().error).toBe("Invalid credentials");
    useAuthStore.getState().clearError();
    expect(useAuthStore.getState().error).toBeNull();
  });

  it("sets loading state", () => {
    useAuthStore.getState().setLoading(true);
    expect(useAuthStore.getState().isLoading).toBe(true);
    useAuthStore.getState().setLoading(false);
    expect(useAuthStore.getState().isLoading).toBe(false);
  });
});
