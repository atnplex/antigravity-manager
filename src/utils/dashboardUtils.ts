import { Account } from '../types/account';

export interface DashboardStats {
    total: number;
    avgGemini: number;
    avgGeminiImage: number;
    avgClaude: number;
    lowQuota: number;
}

export function calculateDashboardStats(accounts: Account[]): DashboardStats {
    let geminiSum = 0;
    let geminiCount = 0;
    let geminiImageSum = 0;
    let geminiImageCount = 0;
    let claudeSum = 0;
    let claudeCount = 0;
    let lowQuotaCount = 0;

    const total = accounts.length;

    for (const account of accounts) {
        if (!account.quota) {
            // Accounts without quota object (shouldn't happen often but possible)
            // Original logic: a.quota?.models... would fail safely (return undefined -> || 0)
            // lowQuotaCount check: a.quota?.is_forbidden. undefined -> false.
            // gemini = undefined -> 0. 0 < 20 -> true.
            // So an account without quota counts as low quota.
            lowQuotaCount++;
            continue;
        }

        let geminiVal = 0;
        let claudeVal = 0;

        // Iterate models once per account
        for (const model of account.quota.models) {
            const name = model.name.toLowerCase();
            if (name === 'gemini-3-pro-high') {
                if (model.percentage > 0) {
                    geminiSum += model.percentage;
                    geminiCount++;
                }
                geminiVal = model.percentage;
            } else if (name === 'gemini-3-pro-image') {
                if (model.percentage > 0) {
                    geminiImageSum += model.percentage;
                    geminiImageCount++;
                }
            } else if (name === 'claude-sonnet-4-5') {
                if (model.percentage > 0) {
                    claudeSum += model.percentage;
                    claudeCount++;
                }
                claudeVal = model.percentage;
            }
        }

        // Low quota check
        if (!account.quota.is_forbidden) {
            // If model is missing, val is 0, which is < 20. Correct.
            if (geminiVal < 20 || claudeVal < 20) {
                lowQuotaCount++;
            }
        }
    }

    return {
        total,
        avgGemini: geminiCount > 0 ? Math.round(geminiSum / geminiCount) : 0,
        avgGeminiImage: geminiImageCount > 0 ? Math.round(geminiImageSum / geminiImageCount) : 0,
        avgClaude: claudeCount > 0 ? Math.round(claudeSum / claudeCount) : 0,
        lowQuota: lowQuotaCount,
    };
}

export interface AccountWithQuota extends Account {
    quotaVal: number;
    geminiQuota?: number;
    claudeQuota?: number;
}

export interface BestAccountsResult {
    bestGemini?: AccountWithQuota;
    bestClaude?: AccountWithQuota;
}

export function getBestAccounts(accounts: Account[], currentAccountId: string | undefined): BestAccountsResult {
    // 1. 获取按配额排序的列表 (排除当前账号)
    // Optimized: Calculate both scores in one pass if possible, but they sort differently.
    // So we need two lists.

    // We can filter once.
    const candidates = currentAccountId
        ? accounts.filter(a => a.id !== currentAccountId)
        : accounts;

    const geminiSorted = candidates
        .map(a => {
            let proQuota = 0;
            let flashQuota = 0;
            if (a.quota?.models) {
                for (const m of a.quota.models) {
                    const name = m.name.toLowerCase();
                    if (name === 'gemini-3-pro-high') proQuota = m.percentage;
                    else if (name === 'gemini-3-flash') flashQuota = m.percentage;
                }
            }
            return {
                ...a,
                quotaVal: Math.round(proQuota * 0.7 + flashQuota * 0.3),
            } as AccountWithQuota;
        })
        .filter(a => a.quotaVal > 0)
        .sort((a, b) => b.quotaVal - a.quotaVal);

    const claudeSorted = candidates
        .map(a => {
            let quotaVal = 0;
            if (a.quota?.models) {
                 for (const m of a.quota.models) {
                    if (m.name.toLowerCase().includes('claude')) {
                        quotaVal = m.percentage;
                        break; // Original used find(), which returns first match
                    }
                }
            }
            return {
                ...a,
                quotaVal,
            } as AccountWithQuota;
        })
        .filter(a => a.quotaVal > 0)
        .sort((a, b) => b.quotaVal - a.quotaVal);

    let bestGemini = geminiSorted[0];
    let bestClaude = claudeSorted[0];

    // 2. 如果推荐是同一个账号，且有其他选择，尝试寻找最优的"不同账号"组合
    if (bestGemini && bestClaude && bestGemini.id === bestClaude.id) {
        const nextGemini = geminiSorted[1];
        const nextClaude = claudeSorted[1];

        // 方案A: 保持 Gemini 最优，换 Claude 次优
        // 方案B: 换 Gemini 次优，保持 Claude 最优

        const scoreA = bestGemini.quotaVal + (nextClaude?.quotaVal || 0);
        const scoreB = (nextGemini?.quotaVal || 0) + bestClaude.quotaVal;

        if (nextClaude && (!nextGemini || scoreA >= scoreB)) {
            // 选方案A：换 Claude
            bestClaude = nextClaude;
        } else if (nextGemini) {
            // 选方案B：换 Gemini
            bestGemini = nextGemini;
        }
    }

    const bestGeminiRender = bestGemini ? { ...bestGemini, geminiQuota: bestGemini.quotaVal } : undefined;
    const bestClaudeRender = bestClaude ? { ...bestClaude, claudeQuota: bestClaude.quotaVal } : undefined;

    return {
        bestGemini: bestGeminiRender,
        bestClaude: bestClaudeRender
    };
}
