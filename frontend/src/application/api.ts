import type { PricingTemplate, QuoteDraft, ToneOption } from "../domain/types";
import { getToken } from "./auth";

const API_BASE: string | undefined = import.meta.env.VITE_API_BASE;
const STORAGE_KEY = "quotesnap:pricing-template";

function hasBackend(): boolean {
  return API_BASE !== undefined && API_BASE !== "";
}

function authHeaders(): Record<string, string> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };

  const token = getToken();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  return headers;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    headers: authHeaders(),
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

export async function getPricingTemplate(): Promise<PricingTemplate | null> {
  if (hasBackend()) {
    return request<PricingTemplate>("/api/pricing");
  }

  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return null;

  return JSON.parse(stored) as PricingTemplate;
}

export async function savePricingTemplate(
  template: PricingTemplate
): Promise<PricingTemplate> {
  if (hasBackend()) {
    return request<PricingTemplate>("/api/pricing", {
      method: "POST",
      body: JSON.stringify(template),
    });
  }

  const toSave: PricingTemplate = {
    ...template,
    id: template.id || "local",
    userId: "local-user",
  };
  localStorage.setItem(STORAGE_KEY, JSON.stringify(toSave));
  return toSave;
}

export async function generateQuote(
  rawText: string,
  imageDataUrls: string[],
  tone: ToneOption
): Promise<QuoteDraft> {
  if (hasBackend()) {
    return request<QuoteDraft>("/api/quote", {
      method: "POST",
      body: JSON.stringify({ rawText, imageDataUrls, tone }),
    });
  }

  // Mock response for local development
  await new Promise((resolve) => setTimeout(resolve, 1500));

  return {
    id: crypto.randomUUID(),
    leadId: crypto.randomUUID(),
    tone,
    jobSummary: {
      serviceType: "General Cleaning",
      propertySize: "3-bedroom house",
      requestedDate: "Next Monday",
      requestedTime: "Morning",
      missingInfo: rawText.length < 50 ? ["Exact address", "Access instructions"] : [],
      extractedDetails: {
        "Service requested": "Deep clean",
        "Property type": "Residential",
        ...(imageDataUrls.length > 0
          ? { "Photos provided": `${imageDataUrls.length} image(s)` }
          : {}),
      },
    },
    estimatedPrice: 185,
    priceBreakdown: [
      { description: "Base cleaning service", amount: 120 },
      { description: "Deep clean surcharge", amount: 45 },
      { description: "Weekend availability", amount: 20 },
    ],
    assumptions: [
      "Standard 3-bedroom layout assumed",
      "No specialist equipment required",
      "Parking available on site",
    ],
    followUpMessage: generateMockFollowUp(tone),
    clarificationMessage:
      rawText.length < 50
        ? "Could you confirm the exact address and how we can access the property?"
        : undefined,
  };
}

function generateMockFollowUp(tone: ToneOption): string {
  switch (tone) {
    case "friendly":
      return `Hi there! Thanks so much for getting in touch. I'd love to help with your cleaning needs!\n\nBased on what you've described, I'd estimate the job at around \u00A3185. This covers a full deep clean of your 3-bedroom home.\n\nHere's a quick breakdown:\n- Base cleaning: \u00A3120\n- Deep clean extras: \u00A345\n- Weekend rate: \u00A320\n\nI'm happy to chat through the details or adjust anything. Just let me know what works for you!`;
    case "direct":
      return `Thank you for your enquiry.\n\nEstimated quote: \u00A3185\n\nBreakdown:\n- Base cleaning: \u00A3120\n- Deep clean surcharge: \u00A345\n- Weekend rate: \u00A320\n\nPlease confirm the address and preferred time slot to proceed with booking.`;
    case "premium":
      return `Good afternoon,\n\nThank you for considering our services. I've prepared a detailed quotation for your property.\n\nYour estimated investment: \u00A3185\n\nThis includes our comprehensive deep cleaning service, tailored to your 3-bedroom residence. Our team uses premium, eco-friendly products and follows a thorough quality checklist.\n\nI'd be delighted to arrange a brief consultation to discuss any specific requirements. Please don't hesitate to reach out at your convenience.`;
  }
}
