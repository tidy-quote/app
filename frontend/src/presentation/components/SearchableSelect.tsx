import { useState, useRef, useEffect, useMemo } from "react";
import "./SearchableSelect.css";

interface Option {
  code: string;
  name: string;
}

interface SearchableSelectProps {
  id: string;
  label: string;
  options: Option[];
  value: string;
  onChange: (code: string) => void;
}

export function SearchableSelect({
  id,
  label,
  options,
  value,
  onChange,
}: SearchableSelectProps): React.JSX.Element {
  const [open, setOpen] = useState(false);
  const [filter, setFilter] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const selected = options.find((o) => o.code === value);
  const displayText = selected ? `${selected.code} — ${selected.name}` : value;

  const filtered = useMemo(() => {
    const q = filter.toLowerCase();
    return options.filter(
      (o) => o.code.toLowerCase().includes(q) || o.name.toLowerCase().includes(q)
    );
  }, [options, filter]);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent): void {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  useEffect(() => {
    if (open) {
      inputRef.current?.focus();
    }
  }, [open]);

  function handleOpen(): void {
    setOpen(true);
    setFilter("");
  }

  function handleSelect(code: string): void {
    onChange(code);
    setOpen(false);
    setFilter("");
  }

  function handleKeyDown(e: React.KeyboardEvent): void {
    if (e.key === "Escape") {
      setOpen(false);
    }
  }

  return (
    <div className="field searchable-select__container" ref={containerRef}>
      <label className="form-label" htmlFor={id}>
        {label}
      </label>
      {open ? (
        <div className="searchable-select" onKeyDown={handleKeyDown}>
          <input
            ref={inputRef}
            id={`${id}-search`}
            className="form-input"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Type to filter..."
            autoComplete="off"
          />
          <ul className="searchable-select__list" role="listbox" aria-label={label}>
            {filtered.map((o) => (
              <li
                key={o.code}
                role="option"
                aria-selected={o.code === value}
                className={`searchable-select__option${o.code === value ? " searchable-select__option--selected" : ""}`}
                onClick={() => handleSelect(o.code)}
              >
                <span className="searchable-select__code">{o.code}</span>
                <span className="searchable-select__name">{o.name}</span>
              </li>
            ))}
            {filtered.length === 0 && (
              <li role="presentation" className="searchable-select__empty">
                No matches
              </li>
            )}
          </ul>
        </div>
      ) : (
        <button
          type="button"
          id={id}
          className="form-input searchable-select__trigger"
          onClick={handleOpen}
        >
          {displayText}
        </button>
      )}
    </div>
  );
}
