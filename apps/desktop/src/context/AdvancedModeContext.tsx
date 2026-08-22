import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

const STORAGE_KEY = "mcgit.advancedMode";

type AdvancedModeContextValue = {
  advancedMode: boolean;
  setAdvancedMode: (value: boolean) => void;
};

const AdvancedModeContext = createContext<AdvancedModeContextValue | null>(null);

export function AdvancedModeProvider({ children }: { children: ReactNode }) {
  const [advancedMode, setAdvancedMode] = useState(() => localStorage.getItem(STORAGE_KEY) === "true");

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, String(advancedMode));
  }, [advancedMode]);

  return (
    <AdvancedModeContext.Provider value={{ advancedMode, setAdvancedMode }}>
      {children}
    </AdvancedModeContext.Provider>
  );
}

export function useAdvancedMode(): AdvancedModeContextValue {
  const context = useContext(AdvancedModeContext);
  if (!context) {
    throw new Error("useAdvancedMode must be used inside an AdvancedModeProvider");
  }
  return context;
}
