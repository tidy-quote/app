import type { Lead, PricingTemplate, QuoteDraft } from "../domain/types";

const API_BASE = import.meta.env.VITE_API_BASE ?? "/api";

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });

  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new Error(
      (body as { error?: string }).error ?? `Request failed: ${response.status}`
    );
  }

  return response.json() as Promise<T>;
}

export function getPricingTemplate(userId: string): Promise<PricingTemplate> {
  return request<PricingTemplate>(`/pricing/${userId}`);
}

export function savePricingTemplate(
  template: PricingTemplate
): Promise<PricingTemplate> {
  return request<PricingTemplate>(`/pricing/${template.id}`, {
    method: "PUT",
    body: JSON.stringify(template),
  });
}

export function submitLead(lead: {
  rawText?: string;
  imageUrls?: string[];
}): Promise<Lead> {
  return request<Lead>("/leads", {
    method: "POST",
    body: JSON.stringify(lead),
  });
}

export function getQuoteDraft(leadId: string): Promise<QuoteDraft> {
  return request<QuoteDraft>(`/quotes/draft/${leadId}`);
}
