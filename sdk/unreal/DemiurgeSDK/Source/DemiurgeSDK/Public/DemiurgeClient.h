// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
// Copyright Demiurge-Cloud. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "DemiurgeTypes.h"
#include "Subsystems/GameInstanceSubsystem.h"
#include "DemiurgeClient.generated.h"

class IWebSocket;

/**
 * Main Demiurge Client - Game Instance Subsystem
 * 
 * Provides connection to the Demiurge-Cloud for:
 * - QOR ID authentication (Sovereign Identity)
 * - DRC-369 asset management (Stateful NFTs)
 * - Optimistic updates for real-time gameplay
 * - Royalty tracking for creator economics
 * 
 * Usage:
 *   UDemiurgeClient* Client = GetGameInstance()->GetSubsystem<UDemiurgeClient>();
 *   Client->Connect("http://127.0.0.1:9944");
 */
UCLASS(BlueprintType)
class DEMIURGESDK_API UDemiurgeClient : public UGameInstanceSubsystem
{
	GENERATED_BODY()

public:
	// =========================================================================
	// SUBSYSTEM LIFECYCLE
	// =========================================================================

	virtual void Initialize(FSubsystemCollectionBase& Collection) override;
	virtual void Deinitialize() override;
	virtual bool ShouldCreateSubsystem(UObject* Outer) const override { return true; }

	// =========================================================================
	// CONNECTION
	// =========================================================================

	/**
	 * Connect to a Demiurge RPC endpoint
	 * @param RpcUrl The RPC endpoint URL (e.g., "http://127.0.0.1:9944")
	 * @param WsUrl Optional WebSocket URL for real-time updates
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Connection")
	void Connect(const FString& RpcUrl, const FString& WsUrl = TEXT(""));

	/**
	 * Disconnect from Demiurge-Cloud
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Connection")
	void Disconnect();

	/**
	 * Get current connection state
	 */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Connection")
	EDemiurgeConnectionState GetConnectionState() const { return ConnectionState; }

	/**
	 * Is connected to the chain
	 */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Connection")
	bool IsConnected() const { return ConnectionState == EDemiurgeConnectionState::Connected; }

	// =========================================================================
	// IDENTITY (QOR ID)
	// =========================================================================

	/**
	 * Authenticate with QOR ID
	 * @param Handle User's handle (e.g., "alice" or "@alice.demiurge")
	 * @param Pin User's PIN
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Identity")
	void Login(const FString& Handle, const FString& Pin);

	/**
	 * Logout and clear session
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Identity")
	void Logout();

	/**
	 * Get the current authenticated identity
	 */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Identity")
	const FQorIdentity& GetCurrentIdentity() const { return CurrentIdentity; }

	/**
	 * Is user authenticated
	 */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Identity")
	bool IsAuthenticated() const { return bIsAuthenticated; }

	/**
	 * Resolve a QOR ID by handle
	 * @param Handle The handle to resolve
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Identity")
	void ResolveIdentity(const FString& Handle);

	// =========================================================================
	// ASSETS (DRC-369)
	// =========================================================================

	/**
	 * Get all assets owned by the current user
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Assets")
	void GetOwnedAssets();

	/**
	 * Get a specific asset by token ID
	 * @param TokenId The 32-byte hex token ID
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Assets")
	void GetAsset(const FString& TokenId);

	/**
	 * Add XP to an asset (requires ownership)
	 * @param TokenId The asset to level up
	 * @param XpAmount Amount of XP to add
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Assets")
	void AddAssetXP(const FString& TokenId, int64 XpAmount);

	/**
	 * Transfer an asset to another user
	 * @param TokenId The asset to transfer
	 * @param ToAddress Recipient's address
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Assets")
	void TransferAsset(const FString& TokenId, const FString& ToAddress);

	/**
	 * Get cached assets (no network call)
	 */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Assets")
	const TArray<FDrc369Asset>& GetCachedAssets() const { return CachedAssets; }

	/**
	 * Find cached asset by token ID
	 * @param TokenId The token ID to find
	 * @param OutAsset The found asset (if successful)
	 * @return True if asset was found
	 */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Assets")
	bool FindCachedAsset(const FString& TokenId, FDrc369Asset& OutAsset) const;

	// =========================================================================
	// OPTIMISTIC UPDATES
	// =========================================================================

	/**
	 * Enable optimistic updates for real-time gameplay
	 * When enabled, UI updates immediately while chain confirms in background
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Optimistic")
	void EnableOptimisticUpdates(bool bEnable);

	/**
	 * Get optimistic state for an asset (pending changes not yet confirmed)
	 */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Optimistic")
	bool GetOptimisticAssetState(const FString& TokenId, FDrc369Asset& OutAsset) const;

	// =========================================================================
	// USAGE TRACKING (for Royalties)
	// =========================================================================

	/**
	 * Report asset usage (render, interaction) for royalty tracking
	 * @param TokenId The asset being used
	 * @param bRendered Was the asset rendered this frame
	 * @param bInteracted Did the player interact with it
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Usage")
	void ReportAssetUsage(const FString& TokenId, bool bRendered, bool bInteracted);

	/**
	 * Flush pending usage reports to the chain
	 * Called automatically, but can be triggered manually
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Usage")
	void FlushUsageReports();

	// =========================================================================
	// CHAIN STATUS
	// =========================================================================

	/**
	 * Get current chain status
	 */
	UFUNCTION(BlueprintPure, Category = "Demiurge|Chain")
	const FDemiurgeChainStatus& GetChainStatus() const { return ChainStatus; }

	/**
	 * Request chain status update
	 */
	UFUNCTION(BlueprintCallable, Category = "Demiurge|Chain")
	void RefreshChainStatus();

	// =========================================================================
	// DELEGATES
	// =========================================================================

	/** Called when identity is ready after login */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnQorIdentityReady OnIdentityReady;

	/** Called when a single asset is loaded */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnDrc369AssetLoaded OnAssetLoaded;

	/** Called when owned assets are loaded */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnDrc369AssetsLoaded OnAssetsLoaded;

	/** Called when a transaction completes */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnTransactionComplete OnTransactionComplete;

	/** Called when connection state changes */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnConnectionStateChanged OnConnectionStateChanged;

	/** Called when chain status is updated */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnChainStatusUpdated OnChainStatusUpdated;

	/** Called when an asset gains XP */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnAssetXPGained OnAssetXPGained;

	/** Called when an asset levels up */
	UPROPERTY(BlueprintAssignable, Category = "Demiurge|Events")
	FOnAssetLevelUp OnAssetLevelUp;

protected:
	// =========================================================================
	// INTERNAL
	// =========================================================================

	/** Make an RPC call */
	void RpcCall(const FString& Method, const TSharedPtr<FJsonObject>& Params, 
		TFunction<void(TSharedPtr<FJsonObject>)> OnSuccess,
		TFunction<void(const FString&)> OnError);

	/** Handle WebSocket message */
	void OnWebSocketMessage(const FString& Message);

	/** Update connection state */
	void SetConnectionState(EDemiurgeConnectionState NewState);

	/** Parse identity from JSON */
	static FQorIdentity ParseIdentity(const TSharedPtr<FJsonObject>& Json);

	/** Parse asset from JSON */
	static FDrc369Asset ParseAsset(const TSharedPtr<FJsonObject>& Json);

private:
	/** RPC endpoint URL */
	FString RpcEndpoint;

	/** WebSocket connection */
	TSharedPtr<IWebSocket> WebSocket;

	/** Current connection state */
	EDemiurgeConnectionState ConnectionState = EDemiurgeConnectionState::Disconnected;

	/** Current authenticated identity */
	FQorIdentity CurrentIdentity;

	/** Is user authenticated */
	bool bIsAuthenticated = false;

	/** Session token */
	FString SessionToken;

	/** Cached assets */
	TArray<FDrc369Asset> CachedAssets;

	/** Optimistic asset states (pending confirmation) */
	TMap<FString, FDrc369Asset> OptimisticStates;

	/** Is optimistic mode enabled */
	bool bOptimisticEnabled = true;

	/** Pending usage reports */
	TMap<FString, TPair<int32, int32>> PendingUsageReports; // TokenId -> (RenderCount, InteractCount)

	/** Chain status */
	FDemiurgeChainStatus ChainStatus;

	/** Timer handle for status polling */
	FTimerHandle StatusPollTimer;

	/** Timer handle for usage flush */
	FTimerHandle UsageFlushTimer;
};
