import SwiftUI

// MARK: - ShareToCircleSheet

/// Quick share: select one or more circles to share an item or all items into.
struct ShareToCircleSheet: View {
    let itemId: UUID?  // nil = share all items, UUID = share specific item
    var alreadySharedCircleIds: Set<UUID> = []
    @Environment(\.dismiss) private var dismiss
    @State private var circles: [OffriiCircle] = []
    @State private var selectedCircleIds: Set<UUID> = []
    @State private var isLoading = false
    @State private var isSharing = false
    @State private var sharedCircleIds: Set<UUID> = []
    @State private var pendingUnshareIds: Set<UUID> = []

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                if isLoading {
                    SkeletonList(count: 4)
                } else if circles.isEmpty {
                    OffriiEmptyState(
                        icon: "person.2.fill",
                        title: NSLocalizedString("share.noCircles", comment: ""),
                        subtitle: NSLocalizedString("share.noCirclesSubtitle", comment: "")
                    )
                } else {
                    ScrollView {
                        VStack(spacing: OffriiTheme.spacingSM) {
                            ForEach(circles) { circle in
                                let isSelected = selectedCircleIds.contains(circle.id)
                                let isShared = sharedCircleIds.contains(circle.id)

                                Button {
                                    if isShared {
                                        // Unshare
                                        sharedCircleIds.remove(circle.id)
                                        pendingUnshareIds.insert(circle.id)
                                    } else if pendingUnshareIds.contains(circle.id) {
                                        // Cancel unshare
                                        pendingUnshareIds.remove(circle.id)
                                        sharedCircleIds.insert(circle.id)
                                    } else if isSelected {
                                        selectedCircleIds.remove(circle.id)
                                    } else {
                                        selectedCircleIds.insert(circle.id)
                                    }
                                } label: {
                                    let isPendingUnshare = pendingUnshareIds.contains(circle.id)
                                    HStack(spacing: OffriiTheme.spacingMD) {
                                        // Circle icon
                                        Image(systemName: circle.isDirect ? "bubble.left.fill" : "person.2.fill")
                                            .font(.system(size: 16))
                                            .foregroundColor(OffriiTheme.primary)
                                            .frame(width: 36, height: 36)
                                            .background(OffriiTheme.primary.opacity(0.1))
                                            .clipShape(Circle())

                                        // Name + member count
                                        VStack(alignment: .leading, spacing: 2) {
                                            Text(circle.name ?? NSLocalizedString("circles.unnamed", comment: ""))
                                                .font(OffriiTypography.body)
                                                .foregroundColor(isPendingUnshare ? OffriiTheme.textMuted : OffriiTheme.text)
                                            Text(String(format: NSLocalizedString("circles.memberCount", comment: ""), circle.memberCount))
                                                .font(OffriiTypography.caption)
                                                .foregroundColor(OffriiTheme.textMuted)
                                        }

                                        Spacer()

                                        // Status
                                        if isPendingUnshare {
                                            Image(systemName: "xmark.circle")
                                                .foregroundColor(OffriiTheme.textMuted)
                                        } else if isShared {
                                            Image(systemName: "checkmark.circle.fill")
                                                .foregroundColor(OffriiTheme.primary)
                                        } else {
                                            Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                                                .foregroundColor(isSelected ? OffriiTheme.primary : OffriiTheme.textMuted)
                                        }
                                    }
                                    .padding(OffriiTheme.spacingBase)
                                    .background(isPendingUnshare ? OffriiTheme.surface : (isSelected ? OffriiTheme.primary.opacity(0.05) : OffriiTheme.card))
                                    .cornerRadius(OffriiTheme.cornerRadiusMD)
                                }
                                .buttonStyle(.plain)
                                .animation(OffriiAnimation.snappy, value: isSelected)
                                .animation(OffriiAnimation.snappy, value: isShared)
                            }
                        }
                        .padding(OffriiTheme.spacingBase)
                    }
                }
            }
            .safeAreaInset(edge: .bottom) {
                if !selectedCircleIds.isEmpty || !pendingUnshareIds.isEmpty {
                    OffriiButton(
                        actionButtonLabel,
                        isLoading: isSharing
                    ) {
                        Task { await applyChanges() }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.bottom, OffriiTheme.spacingSM)
                    .background(OffriiTheme.background)
                }
            }
            .navigationTitle(NSLocalizedString("share.toCircle", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        dismiss()
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }
            .task {
                sharedCircleIds = alreadySharedCircleIds
                isLoading = true
                do {
                    circles = try await CircleService.shared.listCircles()
                } catch {}
                isLoading = false
            }
        }
    }

    private var actionButtonLabel: String {
        let shareCount = selectedCircleIds.count
        let unshareCount = pendingUnshareIds.count
        if shareCount > 0 && unshareCount > 0 {
            return NSLocalizedString("share.applyChanges", comment: "")
        } else if unshareCount > 0 {
            return String(format: NSLocalizedString("share.unshareCount", comment: ""), unshareCount)
        } else {
            return String(format: NSLocalizedString("share.shareToCount", comment: ""), shareCount)
        }
    }

    private func applyChanges() async {
        guard let itemId else { return }
        isSharing = true

        // Unshare
        for circleId in pendingUnshareIds {
            do {
                try await CircleService.shared.unshareItem(circleId: circleId, itemId: itemId)
            } catch {}
        }

        // Share
        for circleId in selectedCircleIds {
            do {
                try await CircleService.shared.shareItem(circleId: circleId, itemId: itemId)
                withAnimation {
                    sharedCircleIds.insert(circleId)
                }
            } catch {}
        }

        OffriiHaptics.success()
        isSharing = false
        selectedCircleIds.removeAll()
        pendingUnshareIds.removeAll()

        try? await Task.sleep(for: .seconds(0.5))
        dismiss()
    }
}
