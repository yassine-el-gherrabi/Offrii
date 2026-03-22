import ContactsUI
import SwiftUI

struct ContactPickerRepresentable: UIViewControllerRepresentable {
    var onSelect: ([String]) -> Void // Returns display names
    var onCancel: () -> Void

    func makeUIViewController(context: Context) -> CNContactPickerViewController {
        let picker = CNContactPickerViewController()
        picker.delegate = context.coordinator
        picker.predicateForEnablingContact = NSPredicate(
            format: "phoneNumbers.@count > 0 OR emailAddresses.@count > 0"
        )
        return picker
    }

    func updateUIViewController(
        _ uiViewController: CNContactPickerViewController,
        context: Context
    ) {}

    func makeCoordinator() -> Coordinator {
        Coordinator(onSelect: onSelect, onCancel: onCancel)
    }

    @MainActor
    final class Coordinator: NSObject, CNContactPickerDelegate {
        let onSelect: ([String]) -> Void
        let onCancel: () -> Void

        init(
            onSelect: @escaping ([String]) -> Void,
            onCancel: @escaping () -> Void
        ) {
            self.onSelect = onSelect
            self.onCancel = onCancel
        }

        nonisolated func contactPicker(
            _ picker: CNContactPickerViewController,
            didSelect contacts: [CNContact]
        ) {
            let names = contacts.map {
                "\($0.givenName) \($0.familyName)".trimmingCharacters(in: .whitespaces)
            }
            MainActor.assumeIsolated {
                onSelect(names)
            }
        }

        nonisolated func contactPickerDidCancel(
            _ picker: CNContactPickerViewController
        ) {
            MainActor.assumeIsolated {
                onCancel()
            }
        }
    }
}
