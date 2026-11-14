// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
// Copyright 2026 Demiurge-Cloud Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
#include "DemiurgeTypes.h"
#include "DemiurgeNetworkManager.generated.h"

// Forward declarations
class IWebSocket;

DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnConnected, bool, bSuccess);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnDisconnected, const FString&, Reason);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnBalanceUpdated, int64, NewBalance);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnQorIDUpdated, const FString&, Username, const FString&, QorKey);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnItemReceived, const FDRC369Item&, Item);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnTradeOfferReceived, const FDemiurgeTradeOffer&, Offer);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnTransactionConfirmed, const FString&, TxHash, bool, bSuccess);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_ThreeParams(FOnChainInfoUpdated, const FString&, ChainName, const FString&, Version, int32, BlockNumber);

/**
 * Demiurge Network Manager - Substrate RPC Bridge
 * 
 * This class manages the WebSocket connection to the Demiurge Substrate node.
 * It handles all blockchain interactions:
 * - CGT balance queries and transfers
 * - Qor ID registration and lookups
 * - DRC-369 item queries and trades
 * 
 * Connection:
 *   Default: ws://127.0.0.1:9944 (local node)
 *   Production: ws://127.0.0.1:9944
 */
UCLASS(Blueprintable, BlueprintType)
class DEMIURGEWEB3_API UDemiurgeNetworkManager : public UObject
{
	GENERATED_BODY()

public:
	UDemiurgeNetworkManager();

	// ═══════════════════════════════════════════════════════════════════════════
	// CONNECTION
	// ═══════════════════════════════════════════════════════════════════════════

	/** Connect to the Demiurge node */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Network")
	void Connect(const FString& NodeURL = TEXT("ws://127.0.0.1:9944"));

	/** Disconnect from the node */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Network")
	void Disconnect();

	/** Check if currently connected */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Network")
	bool IsConnected() const { return bIsConnected; }

	/** Get the current node URL */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Network")
	FString GetNodeURL() const { return CurrentNodeURL; }

	/** Enable/disable auto-reconnection */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Network")
	void SetAutoReconnect(bool bEnabled, float DelaySeconds = 5.0f);

	// ═══════════════════════════════════════════════════════════════════════════
	// CGT (Creator God Token)
	// ═══════════════════════════════════════════════════════════════════════════

	/** Get CGT balance for an account */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|CGT")
	void GetCGTBalance(const FString& AccountAddress);

	/** Transfer CGT to another account */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|CGT")
	void TransferCGT(const FString& ToAddress, int64 Amount);

	/** Get total CGT burned since genesis */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|CGT")
	void GetTotalBurned();

	// ═══════════════════════════════════════════════════════════════════════════
	// QOR ID (Identity)
	// ═══════════════════════════════════════════════════════════════════════════

	/** Register a new Qor ID (costs 5 CGT) */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|QorID")
	void RegisterQorID(const FString& Username);

	/** Lookup Qor ID by account address */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|QorID")
	void LookupQorID(const FString& AccountAddress);

	/** Check if a username is available */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|QorID")
	void CheckUsernameAvailability(const FString& Username);

	// ═══════════════════════════════════════════════════════════════════════════
	// DRC-369 (Items)
	// ═══════════════════════════════════════════════════════════════════════════

	/** Get all items owned by an account */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|DRC369")
	void GetInventory(const FString& AccountAddress);

	/** Get item details by UUID */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|DRC369")
	void GetItem(const FString& ItemUUID);

	/** Initiate a trade offer */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|DRC369")
	void InitiateTrade(const FString& ItemUUID, const FString& ReceiverAddress);

	/** Accept a trade offer */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|DRC369")
	void AcceptTrade(const FString& OfferID);

	/** Cancel a trade offer */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|DRC369")
	void CancelTrade(const FString& OfferID);

	// ═══════════════════════════════════════════════════════════════════════════
	// SUBSTRATE SYSTEM RPC
	// ═══════════════════════════════════════════════════════════════════════════

	/** Get chain name and version */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|System")
	void GetChainInfo();

	/** Subscribe to new block headers */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|System")
	void SubscribeNewHeads();

	/** Get current block number */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|System")
	void GetBlockNumber();

	// ═══════════════════════════════════════════════════════════════════════════
	// EVENTS
	// ═══════════════════════════════════════════════════════════════════════════

	/** Fired when connection succeeds or fails */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnConnected OnConnected;

	/** Fired when disconnected */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnDisconnected OnDisconnected;

	/** Fired when CGT balance is updated */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnBalanceUpdated OnBalanceUpdated;

	/** Fired when Qor ID is resolved */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnQorIDUpdated OnQorIDUpdated;

	/** Fired when a new item is received (minted or traded) */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnItemReceived OnItemReceived;

	/** Fired when a trade offer is received */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnTradeOfferReceived OnTradeOfferReceived;

	/** Fired when a transaction is confirmed */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnTransactionConfirmed OnTransactionConfirmed;

	/** Fired when chain info is updated */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnChainInfoUpdated OnChainInfoUpdated;

protected:
	/** Current node URL */
	FString CurrentNodeURL;

	/** Connection state */
	bool bIsConnected;

	/** WebSocket connection handle */
	TSharedPtr<class IWebSocket> WebSocket;

	/** Request ID counter for JSON-RPC */
	int32 NextRequestID;

	/** Map of pending requests (RequestID -> Method) */
	TMap<int32, FString> PendingRequests;

	/** Reconnection timer handle */
	FTimerHandle ReconnectTimerHandle;

	/** Auto-reconnect enabled */
	bool bAutoReconnect;

	/** Reconnection delay in seconds */
	float ReconnectDelay;

	/** Process incoming JSON-RPC message */
	void ProcessMessage(const FString& Message);

	/** Send JSON-RPC request and return request ID */
	int32 SendRPCRequest(const FString& Method, const TArray<FString>& Params);

	/** Send JSON-RPC request with custom params object */
	int32 SendRPCRequestWithParams(const FString& Method, const TSharedPtr<FJsonObject>& ParamsObject);

	/** Handle RPC response by request ID */
	void HandleRPCResponse(int32 RequestID, const TSharedPtr<FJsonObject>& ResultObject);

	/** Handle RPC error */
	void HandleRPCError(int32 RequestID, const TSharedPtr<FJsonObject>& ErrorObject);

	/** Attempt reconnection */
	void AttemptReconnection();

	/** Handle connection opened */
	void OnWebSocketConnected();

	/** Handle connection closed */
	void OnWebSocketClosed(int32 StatusCode, const FString& Reason, bool bWasClean);

	/** Handle message received */
	void OnWebSocketMessage(const FString& Message);
};
