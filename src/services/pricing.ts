export function formatCurrency(value: number | null | undefined, currency = "$") {
  if (value === null || value === undefined || Number.isNaN(value)) {
    return "-";
  }
  return `${currency}${value.toFixed(6)}`;
}

export function formatPercent(value: number | null | undefined) {
  if (value === null || value === undefined || Number.isNaN(value)) {
    return "-";
  }
  return `${(value * 100).toFixed(2)}%`;
}

export function calculateActualCost(balanceBefore: number, balanceAfter: number) {
  return Math.max(0, balanceBefore - balanceAfter);
}

export function calculateDiffRatio(actualCost: number, estimatedCost: number) {
  if (estimatedCost <= 0) {
    return null;
  }
  return (actualCost - estimatedCost) / estimatedCost;
}
