import SwiftUI

// MARK: - Sort Menu (reusable across Envies & Entraide)

struct SortMenuView: View {
    let options: [(key: String, label: String)]
    @Binding var sortField: String
    @Binding var sortOrder: String

    private var currentLabel: String {
        options.first(where: { $0.key == sortField })?.label ?? options.first?.label ?? ""
    }

    var body: some View {
        Menu {
            ForEach(options, id: \.key) { option in
                Button {
                    if sortField == option.key {
                        sortOrder = sortOrder == "desc" ? "asc" : "desc"
                    } else {
                        sortField = option.key
                        sortOrder = option.key == "name" || option.key == "title" ? "asc" : "desc"
                    }
                } label: {
                    HStack {
                        Text(option.label)
                        if sortField == option.key {
                            Image(systemName: sortOrder == "desc" ? "chevron.down" : "chevron.up")
                        }
                    }
                }
            }
        } label: {
            HStack(spacing: 2) {
                Text(currentLabel)
                    .font(.system(size: 13, weight: .medium))
                Image(systemName: sortOrder == "desc" ? "chevron.down" : "chevron.up")
                    .font(.system(size: 10, weight: .semibold))
            }
            .foregroundColor(OffriiTheme.primary)
        }
    }
}
