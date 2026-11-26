# Core Gameplay Systems

## System overview
| System | Responsibility | Dependencies |
|---|---|---|
| Company | Identitas dan status perusahaan | Finance, Product, Employee |
| Product | Planning, development, quality, lifecycle | Employee, Time, Market |
| Employee | Skill, energy, happiness, productivity | Company, Product, Time |
| Finance | Cashflow, costs, revenue, risk | Company, Product, Employee, Market |
| Market | Demand, competition, trends, sales | Product, Finance, Reputation |
| Office | Space, furniture, equipment, layout | Interaction, Company |
| Event | Dynamic events and consequences | Time, Market, Finance, Employee |
| Time | Clock and simulation ticks | Time-dependent systems |
| Interaction | First-person object interaction | Player, Office, UI |
| UI | Management screens and information | Relevant systems |
| Save | Persistence and restore | Persistent data |

## System boundaries
- Simulasi bisnis tidak bergantung pada mesh atau asset visual.
- Visual entity merepresentasikan data/state, bukan menjadi sumber kebenaran gameplay.
- UI membaca state dan mengirim command; bukan sumber state bisnis utama.
- Event dan perubahan state terstruktur agar dapat diuji.
- Hindari systems saling terikat berlebihan.

## MVP integration order
1. Time + Company + Finance basics.
2. Product development.
3. Employee productivity.
4. Market sales.
5. Management UI.
6. Save/load.
7. Office expansion and polish.
