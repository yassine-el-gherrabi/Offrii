import SwiftUI

/// A sheet that lets the user pick specific items to share.
struct ItemPickerSheet: View {
    let items: [Item]
    @Binding var selectedIds: Set<UUID>
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                ForEach(items) { item in
                    let isSelected = selectedIds.contains(item.id)

                    Button {
                        if isSelected {
                            selectedIds.remove(item.id)
                        } else {
                            selectedIds.insert(item.id)
                        }
                    } label: {
                        HStack(spacing: OffriiTheme.spacingMD) {
                            Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                                .font(.system(size: 20))
                                .foregroundColor(isSelected ? OffriiTheme.primary : OffriiTheme.textMuted)

                            VStack(alignment: .leading, spacing: 2) {
                                Text(item.name)
                                    .font(OffriiTypography.body)
                                    .foregroundColor(OffriiTheme.text)

                                if let price = item.estimatedPrice {
                                    let formatter = NumberFormatter()
                                    let _ = { formatter.numberStyle = .currency; formatter.currencyCode = "EUR" }()
                                    Text(formatter.string(from: price as NSDecimalNumber) ?? "")
                                        .font(OffriiTypography.caption)
                                        .foregroundColor(OffriiTheme.textMuted)
                                }
                            }

                            Spacer()
                        }
                    }
                    .listRowBackground(OffriiTheme.card)
                }
            }
            .listStyle(.plain)
            .navigationTitle(NSLocalizedString("share.pickItems", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button(NSLocalizedString("common.ok", comment: "")) {
                        dismiss()
                    }
                    .fontWeight(.semibold)
                }

                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        dismiss()
                    }
                }
            }
            .safeAreaInset(edge: .bottom) {
                if !selectedIds.isEmpty {
                    Text(String(format: NSLocalizedString("share.itemsSelected", comment: ""), selectedIds.count))
                        .font(.system(size: 13, weight: .medium))
                        .foregroundColor(OffriiTheme.primary)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, OffriiTheme.spacingSM)
                        .background(.ultraThinMaterial)
                }
            }
        }
    }
}
