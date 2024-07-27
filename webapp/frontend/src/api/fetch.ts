class Fetch {
  private static baseURL: string = process.env.API_BASE_URL || "http://nginx";

  private static instance: Fetch;

  public static getInstance(): Fetch {
    if (!Fetch.instance) {
      Fetch.instance = new Fetch();
    }
    return Fetch.instance;
  }

  public async fetch<T>(endpoint: string, options?: RequestInit): Promise<T> {
    try {
      const response = await fetch(`${Fetch.baseURL}${endpoint}`, { cache: "no-cache", ...options });

      if (!response.ok) {
        throw new Error(`Fetch request failed with status ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Fetch error:", error);
      throw error;
    }
  }
}

export default Fetch;
