export type ApiErrorPayload = {
  error?: {
    message?: string;
    status?: number;
  };
};

export type User = {
  id: string;
  name: string;
  email: string;
  enabled: boolean;
};

export type AuthResponse = {
  user: User;
  token: string;
};

export type Provider = {
  id: string;
  provider: string;
  label: string;
  tags: string[];
  enabled: boolean;
  last_used_at: string | null;
  sync_status: string;
  sync_error: string | null;
  last_synced_at: string | null;
  model_count: number;
  created_at: string;
  updated_at: string;
};

export type VirtualKeyRoute = {
  model_alias: string;
  provider_credential_id: string;
  provider: string;
  provider_label: string;
};

export type VirtualKey = {
  id: string;
  name: string;
  key_prefix: string;
  allowed_models: string[];
  model_routes: VirtualKeyRoute[];
  tags: string[];
  enabled: boolean;
  expires_at: string | null;
  last_used_at: string | null;
  created_at: string;
  updated_at: string;
};

export type CreateVirtualKeyResponse = {
  key: VirtualKey;
  raw_key: string;
};

export type Model = {
  id: string;
  alias: string;
  display_name: string;
  provider: string;
  upstream_model: string;
  description: string | null;
  enabled: boolean;
  context_window_tokens: number;
  max_output_tokens: number;
  supports_streaming: boolean;
  supports_thinking: boolean;
  thinking_required: boolean;
  supports_temperature: boolean;
  temperature_fixed_to: number | null;
  temperature_min: number | null;
  temperature_max: number | null;
  supports_top_p: boolean;
  supports_system_messages: boolean;
  supports_tools: boolean;
  supports_vision: boolean;
  supports_json_mode: boolean;
  supports_parallel_tool_calls: boolean;
};

export type ProviderInput = {
  provider: "google-genai";
  label: string;
  api_key: string;
  tags: string[];
};

export type ProviderUpdateInput = {
  label: string;
  api_key?: string;
  tags: string[];
  enabled: boolean;
};

export type VirtualKeyRouteInput = {
  model_alias: string;
  provider_credential_id: string;
};

export type VirtualKeyInput = {
  name: string;
  allowed_models: string[];
  model_routes: VirtualKeyRouteInput[];
  tags: string[];
  expires_at: string | null;
};

export type VirtualKeyUpdateInput = VirtualKeyInput & {
  enabled: boolean;
};

export type FieldState = {
  success?: string;
  error?: string;
  rawKey?: string;
  createdKeyId?: string;
};
