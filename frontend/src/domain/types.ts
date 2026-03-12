export interface AuthUser {
  id: string;
  email: string;
}

export interface AuthState {
  user: AuthUser | null;
  token: string | null;
}

export interface ServiceCategory {
  id: string;
  name: string;
  basePrice: number;
  description: string;
}

export interface AddOn {
  id: string;
  name: string;
  price: number;
}

export interface PricingTemplate {
  id: string;
  userId: string;
  currency: string;
  country: string;
  minimumCallout: number;
  categories: ServiceCategory[];
  addOns: AddOn[];
  customNotes: string;
}

export type ToneOption = "friendly" | "direct" | "premium";

export interface Lead {
  id: string;
  rawText?: string;
  imageUrls?: string[];
  createdAt: string;
}

export interface JobSummary {
  serviceType: string;
  propertySize?: string;
  requestedDate?: string;
  requestedTime?: string;
  missingInfo: string[];
  extractedDetails: Record<string, string>;
}

export interface PriceLineItem {
  description: string;
  amount: number;
}

export interface QuoteDraft {
  id: string;
  leadId: string;
  jobSummary: JobSummary;
  estimatedPrice: number;
  priceBreakdown: PriceLineItem[];
  assumptions: string[];
  followUpMessage: string;
  clarificationMessage?: string;
  tone: ToneOption;
}
