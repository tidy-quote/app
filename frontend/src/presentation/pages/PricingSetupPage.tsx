import { useState, type FormEvent } from "react";
import type { ServiceCategory, AddOn } from "../../domain/types";
import "./PricingSetupPage.css";

const EMPTY_CATEGORY: ServiceCategory = {
  id: "",
  name: "",
  basePrice: 0,
  description: "",
};

const EMPTY_ADDON: AddOn = {
  id: "",
  name: "",
  price: 0,
};

export function PricingSetupPage(): React.JSX.Element {
  const [currency, setCurrency] = useState("GBP");
  const [country, setCountry] = useState("UK");
  const [minimumCallout, setMinimumCallout] = useState(0);
  const [categories, setCategories] = useState<ServiceCategory[]>([
    { ...EMPTY_CATEGORY, id: "1" },
  ]);
  const [addOns, setAddOns] = useState<AddOn[]>([
    { ...EMPTY_ADDON, id: "1" },
  ]);
  const [customNotes, setCustomNotes] = useState("");

  function updateCategory(
    index: number,
    field: keyof ServiceCategory,
    value: string | number
  ): void {
    setCategories((prev) =>
      prev.map((cat, i) => (i === index ? { ...cat, [field]: value } : cat))
    );
  }

  function addCategory(): void {
    setCategories((prev) => [
      ...prev,
      { ...EMPTY_CATEGORY, id: String(Date.now()) },
    ]);
  }

  function removeCategory(index: number): void {
    setCategories((prev) => prev.filter((_, i) => i !== index));
  }

  function updateAddOn(
    index: number,
    field: keyof AddOn,
    value: string | number
  ): void {
    setAddOns((prev) =>
      prev.map((a, i) => (i === index ? { ...a, [field]: value } : a))
    );
  }

  function addAddOn(): void {
    setAddOns((prev) => [...prev, { ...EMPTY_ADDON, id: String(Date.now()) }]);
  }

  function removeAddOn(index: number): void {
    setAddOns((prev) => prev.filter((_, i) => i !== index));
  }

  function handleSubmit(e: FormEvent): void {
    e.preventDefault();
    // TODO: wire up to API
    console.log("Saving pricing template:", {
      currency,
      country,
      minimumCallout,
      categories,
      addOns,
      customNotes,
    });
  }

  return (
    <div className="pricing-setup">
      <h2 className="page-title">Pricing Setup</h2>
      <p className="page-desc">
        Configure your service categories, add-ons, and default rates.
      </p>

      <form onSubmit={handleSubmit} className="pricing-form">
        <fieldset className="form-section">
          <legend className="section-title">General</legend>

          <div className="field-row">
            <div className="field">
              <label className="form-label" htmlFor="currency">
                Currency
              </label>
              <input
                id="currency"
                className="form-input"
                value={currency}
                onChange={(e) => setCurrency(e.target.value)}
              />
            </div>
            <div className="field">
              <label className="form-label" htmlFor="country">
                Country
              </label>
              <input
                id="country"
                className="form-input"
                value={country}
                onChange={(e) => setCountry(e.target.value)}
              />
            </div>
          </div>

          <div className="field">
            <label className="form-label" htmlFor="min-callout">
              Minimum callout ({currency})
            </label>
            <input
              id="min-callout"
              type="number"
              min={0}
              className="form-input"
              value={minimumCallout}
              onChange={(e) => setMinimumCallout(Number(e.target.value))}
            />
          </div>
        </fieldset>

        <fieldset className="form-section">
          <legend className="section-title">Service Categories</legend>
          {categories.map((cat, i) => (
            <div key={cat.id} className="list-item">
              <input
                className="form-input"
                placeholder="Category name"
                value={cat.name}
                onChange={(e) => updateCategory(i, "name", e.target.value)}
              />
              <input
                className="form-input form-input--short"
                type="number"
                min={0}
                placeholder="Price"
                value={cat.basePrice || ""}
                onChange={(e) =>
                  updateCategory(i, "basePrice", Number(e.target.value))
                }
              />
              <input
                className="form-input"
                placeholder="Description"
                value={cat.description}
                onChange={(e) =>
                  updateCategory(i, "description", e.target.value)
                }
              />
              <button
                type="button"
                className="btn-remove"
                onClick={() => removeCategory(i)}
                aria-label="Remove category"
              >
                Remove
              </button>
            </div>
          ))}
          <button type="button" className="btn-secondary" onClick={addCategory}>
            + Add Category
          </button>
        </fieldset>

        <fieldset className="form-section">
          <legend className="section-title">Add-Ons</legend>
          {addOns.map((addon, i) => (
            <div key={addon.id} className="list-item">
              <input
                className="form-input"
                placeholder="Add-on name"
                value={addon.name}
                onChange={(e) => updateAddOn(i, "name", e.target.value)}
              />
              <input
                className="form-input form-input--short"
                type="number"
                min={0}
                placeholder="Price"
                value={addon.price || ""}
                onChange={(e) =>
                  updateAddOn(i, "price", Number(e.target.value))
                }
              />
              <button
                type="button"
                className="btn-remove"
                onClick={() => removeAddOn(i)}
                aria-label="Remove add-on"
              >
                Remove
              </button>
            </div>
          ))}
          <button type="button" className="btn-secondary" onClick={addAddOn}>
            + Add-On
          </button>
        </fieldset>

        <fieldset className="form-section">
          <legend className="section-title">Custom Notes</legend>
          <textarea
            className="form-textarea"
            rows={3}
            placeholder="Any additional notes for quote generation..."
            value={customNotes}
            onChange={(e) => setCustomNotes(e.target.value)}
          />
        </fieldset>

        <button type="submit" className="btn-primary">
          Save Pricing Template
        </button>
      </form>
    </div>
  );
}
