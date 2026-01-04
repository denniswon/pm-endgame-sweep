"use client";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  formatPercent,
  formatDuration,
  formatScore,
  getRiskColor,
  getSeverityColor,
} from "@/lib/utils";
import type { MarketDetailsResponse, RiskFlag } from "@/types/api";
import { Calendar, TrendingUp, AlertTriangle } from "lucide-react";

export function MarketCard({ data }: { data: MarketDetailsResponse }) {
  const { market, quote, score, recommendation } = data;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between">
            <div>
              <CardTitle className="text-2xl">{market.question}</CardTitle>
              <CardDescription className="mt-2">
                Market ID: {market.market_id}
              </CardDescription>
            </div>
            <Badge variant={market.active ? "default" : "secondary"}>
              {market.active ? "Active" : "Inactive"}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          {market.description && (
            <p className="text-sm text-muted-foreground mb-4">
              {market.description}
            </p>
          )}
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="flex items-center gap-2">
              <Calendar className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">Close Time:</span>
              <span className="font-medium">
                {new Date(market.close_time).toLocaleString()}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <TrendingUp className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">Volume:</span>
              <span className="font-medium">
                ${market.volume.toLocaleString()}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {quote && (
        <Card>
          <CardHeader>
            <CardTitle>Current Quotes</CardTitle>
            <CardDescription>
              As of {new Date(quote.as_of).toLocaleString()}
              {quote.staleness_sec > 0 && (
                <span className="ml-2 text-yellow-600">
                  ({formatDuration(quote.staleness_sec)} stale)
                </span>
              )}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-green-600">YES</h4>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div>
                    <span className="text-muted-foreground">Bid:</span>
                    <span className="ml-2 font-mono">
                      {quote.yes_bid !== null
                        ? formatPercent(quote.yes_bid)
                        : "N/A"}
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Ask:</span>
                    <span className="ml-2 font-mono">
                      {quote.yes_ask !== null
                        ? formatPercent(quote.yes_ask)
                        : "N/A"}
                    </span>
                  </div>
                </div>
              </div>
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-red-600">NO</h4>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div>
                    <span className="text-muted-foreground">Bid:</span>
                    <span className="ml-2 font-mono">
                      {quote.no_bid !== null
                        ? formatPercent(quote.no_bid)
                        : "N/A"}
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Ask:</span>
                    <span className="ml-2 font-mono">
                      {quote.no_ask !== null
                        ? formatPercent(quote.no_ask)
                        : "N/A"}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {score && (
        <Card>
          <CardHeader>
            <CardTitle>Score Breakdown</CardTitle>
            <CardDescription>
              As of {new Date(score.as_of).toLocaleString()}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Overall Score</span>
                <span className="text-lg font-bold text-primary">
                  {formatScore(score.overall_score)}
                </span>
              </div>
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Yield Velocity:</span>
                  <span className="font-mono">
                    {formatScore(score.yield_velocity)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Net Yield:</span>
                  <span className="font-mono">
                    {formatPercent(score.net_yield)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">
                    Liquidity Score:
                  </span>
                  <span className="font-mono">
                    {formatScore(score.liquidity_score)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">
                    Definition Risk:
                  </span>
                  <span className="font-mono">
                    {formatScore(score.definition_risk_score)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Staleness:</span>
                  <span className="font-mono">
                    {formatScore(score.staleness_penalty)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Time Remaining:</span>
                  <span className="font-mono">
                    {formatDuration(score.t_remaining_sec)}
                  </span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {recommendation && (
        <Card>
          <CardHeader>
            <CardTitle>Recommendation</CardTitle>
            <CardDescription>
              Generated {new Date(recommendation.as_of).toLocaleString()}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center gap-4">
                <div>
                  <span className="text-sm text-muted-foreground">
                    Recommended Side:
                  </span>
                  <Badge
                    className="ml-2"
                    variant={
                      recommendation.recommended_side === "YES"
                        ? "default"
                        : "secondary"
                    }
                  >
                    {recommendation.recommended_side}
                  </Badge>
                </div>
                <div>
                  <span className="text-sm text-muted-foreground">
                    Risk Score:
                  </span>
                  <span
                    className={`ml-2 inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ${getRiskColor(
                      recommendation.risk_score
                    )}`}
                  >
                    {formatScore(recommendation.risk_score)}
                  </span>
                </div>
              </div>
              <div className="grid grid-cols-3 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Entry Price:</span>
                  <p className="font-mono font-medium">
                    {formatPercent(recommendation.entry_price)}
                  </p>
                </div>
                <div>
                  <span className="text-muted-foreground">
                    Expected Payout:
                  </span>
                  <p className="font-mono font-medium">
                    {formatPercent(recommendation.expected_payout)}
                  </p>
                </div>
                <div>
                  <span className="text-muted-foreground">Max Position:</span>
                  <p className="font-mono font-medium">
                    {formatPercent(recommendation.max_position_pct)}
                  </p>
                </div>
              </div>
              {recommendation.risk_flags &&
                recommendation.risk_flags.length > 0 && (
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <AlertTriangle className="h-4 w-4 text-yellow-600" />
                      <span>
                        Risk Flags ({recommendation.risk_flags.length})
                      </span>
                    </div>
                    <div className="space-y-1">
                      {recommendation.risk_flags.map(
                        (flag: RiskFlag, i: number) => (
                          <div
                            key={i}
                            className="flex items-center gap-2 text-sm"
                          >
                            <Badge
                              className={getSeverityColor(
                                flag.severity || "low"
                              )}
                            >
                              {flag.code || "UNKNOWN"}
                            </Badge>
                            {flag.severity && (
                              <span className="text-muted-foreground text-xs">
                                Severity: {flag.severity}
                              </span>
                            )}
                          </div>
                        )
                      )}
                    </div>
                  </div>
                )}
              {recommendation.notes && (
                <div className="pt-2 border-t">
                  <span className="text-sm text-muted-foreground">Notes:</span>
                  <p className="mt-1 text-sm">{recommendation.notes}</p>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
